use std::io;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use sci_task_io::trajectory::TrajectoryHub;

use crate::progress::{ProgressEvent, ProgressStore, new_progress_store, ProgressHandle};
use crate::task::{Task, TaskContext};

/// Manager-owned runtime settings for one task group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskGroupConfig {
    /// Number of writer threads owned by the shared trajectory hub.
    pub num_writer_threads: usize,
    /// Maximum number of threads each task may use internally.
    pub task_num_threads: usize,
    /// Optional cap on how many tasks may advance concurrently.
    pub num_task_threads: Option<usize>,
}

/// Error returned while creating or reconfiguring task-group runtime state.
#[derive(Debug)]
pub enum TaskGroupInitError {
    /// Failed to start the shared trajectory hub.
    Hub(io::Error),
    /// Failed to build one of the Rayon thread pools.
    ThreadPool(ThreadPoolBuildError),
}

impl From<io::Error> for TaskGroupInitError {
    /// Wrap a trajectory-hub startup error as a task-group init error.
    fn from(value: io::Error) -> Self {
        Self::Hub(value)
    }
}

impl From<ThreadPoolBuildError> for TaskGroupInitError {
    /// Wrap a Rayon pool build error as a task-group init error.
    fn from(value: ThreadPoolBuildError) -> Self {
        Self::ThreadPool(value)
    }
}

/// Generic task group that owns tasks, a shared trajectory hub, and progress state.
pub struct TaskGroup<T: Task> {
    tasks: Vec<T>,
    hub: TrajectoryHub,
    progress_store: ProgressStore,
    progress_tx: std::sync::mpsc::Sender<ProgressEvent>,
    config: TaskGroupConfig,
    group_pool: ThreadPool,
    task_pools: Vec<Arc<ThreadPool>>,
    epochs_run: u64,
}

impl<T: Task> TaskGroup<T> {
    /// Create a new task group from manager-owned runtime configuration.
    pub fn new(config: TaskGroupConfig) -> Result<Self, TaskGroupInitError> {
        let (progress_tx, progress_store) = new_progress_store();
        let hub = TrajectoryHub::start(config.num_writer_threads)?;
        let group_pool = Self::build_group_pool(config.num_task_threads, 0)?;
        Ok(Self {
            tasks: Vec::new(),
            hub,
            progress_store,
            progress_tx,
            config,
            group_pool,
            task_pools: Vec::new(),
            epochs_run: 0,
        })
    }

    /// Build the Rayon pool responsible for scheduling tasks within one epoch.
    fn build_group_pool(
        num_task_threads: Option<usize>,
        num_tasks: usize,
    ) -> Result<ThreadPool, ThreadPoolBuildError> {
        let group_pool_size = num_task_threads.unwrap_or(num_tasks.max(1));
        ThreadPoolBuilder::new()
            .num_threads(group_pool_size)
            .thread_name(|idx| format!("sci-task-group-{idx}"))
            .build()
    }

    /// Build one Rayon pool dedicated to internal parallel work for one task.
    fn build_task_pool(
        task_index: usize,
        task_num_threads: usize,
    ) -> Result<Arc<ThreadPool>, ThreadPoolBuildError> {
        ThreadPoolBuilder::new()
            .num_threads(task_num_threads)
            .thread_name(move |idx| format!("sci-task-{task_index}-{idx}"))
            .build()
            .map(Arc::new)
    }

    /// Append one task to the group.
    pub fn add_task(&mut self, task: T) -> Result<(), TaskGroupInitError> {
        self.add_tasks(std::iter::once(task))
    }

    /// Append many tasks to the group.
    pub fn add_tasks<I>(&mut self, tasks: I) -> Result<(), TaskGroupInitError>
    where
        I: IntoIterator<Item = T>,
    {
        for task in tasks {
            let task_index = self.tasks.len();
            self.task_pools
                .push(Self::build_task_pool(task_index, self.config.task_num_threads)?);
            self.tasks.push(task);
        }

        self.group_pool = Self::build_group_pool(self.config.num_task_threads, self.tasks.len())?;
        Ok(())
    }

    /// Return the number of tasks in the group.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Return whether the group contains no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Return the number of epochs the group has fully executed.
    pub fn epochs_run(&self) -> u64 {
        self.epochs_run
    }

    /// Return the shared trajectory hub owned by the group.
    pub fn hub(&self) -> &TrajectoryHub {
        &self.hub
    }

    /// Drain queued progress events into the in-memory store.
    pub fn drain_progress(&mut self) {
        self.progress_store.drain();
    }

    /// Return the collected progress events accumulated so far.
    pub fn progress_events(&self) -> &[ProgressEvent] {
        self.progress_store.snapshot()
    }

    /// Return the maximum number of threads each task may use internally.
    pub fn task_num_threads(&self) -> usize {
        self.config.task_num_threads
    }

    /// Return the configured cap on concurrently advancing tasks, if any.
    pub fn num_task_threads(&self) -> Option<usize> {
        self.config.num_task_threads
    }

    /// Return the number of writer threads owned by the internal trajectory hub.
    pub fn num_writer_threads(&self) -> usize {
        self.config.num_writer_threads
    }

    /// Return an immutable slice of the tasks owned by the group.
    pub fn tasks(&self) -> &[T] {
        &self.tasks
    }

    /// Return a mutable slice of the tasks owned by the group.
    pub fn tasks_mut(&mut self) -> &mut [T] {
        &mut self.tasks
    }

    /// Consume the group and return the owned tasks without shutting down the hub.
    pub fn into_tasks(self) -> Vec<T> {
        self.tasks
    }

    /// Run exactly one epoch for every task in the group.
    pub fn run_one_epoch(&mut self) -> Result<(), T::Error> {
        let epoch = self.epochs_run + 1;
        let hub = &self.hub;
        let progress_tx = self.progress_tx.clone();
        let task_num_threads = self.config.task_num_threads;
        let task_pools = &self.task_pools;
        let tasks = &mut self.tasks;

        self.group_pool.install(|| {
            tasks
                .par_iter_mut()
                .enumerate()
                .try_for_each(|(task_index, task)| {
                    let progress = ProgressHandle::new(task_index, epoch, progress_tx.clone());
                    let context = TaskContext::new(
                        hub,
                        &progress,
                        epoch,
                        task_num_threads,
                        Arc::clone(&task_pools[task_index]),
                    );
                    task.evolve_one_epoch(&context)
                })
        })?;

        self.epochs_run = epoch;
        Ok(())
    }

    /// Run `num_epochs` consecutive epochs across the whole group.
    pub fn run_epochs(&mut self, num_epochs: u64) -> Result<(), T::Error> {
        for _ in 0..num_epochs {
            self.run_one_epoch()?;
        }
        Ok(())
    }

    /// Shut down the shared hub and return the owned tasks.
    pub fn shutdown(self) -> io::Result<Vec<T>> {
        let Self {
            tasks,
            hub,
            progress_store: _,
            progress_tx: _,
            config: _,
            group_pool: _,
            task_pools: _,
            epochs_run: _,
        } = self;
        hub.shutdown()?;
        Ok(tasks)
    }
}
