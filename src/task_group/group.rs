use std::io;

use sci_task_io::trajectory::TrajectoryHub;

use crate::progress::{ProgressEvent, ProgressStore, new_progress_store, ProgressHandle};
use crate::task::{Task, TaskContext};

/// Generic task group that owns tasks, a shared trajectory hub, and progress state.
pub struct TaskGroup<T: Task> {
    tasks: Vec<T>,
    hub: TrajectoryHub,
    progress_store: ProgressStore,
    progress_tx: std::sync::mpsc::Sender<ProgressEvent>,
    epochs_run: u64,
}

impl<T: Task> TaskGroup<T> {
    /// Create a new task group from a task list and a shared trajectory hub.
    pub fn new(tasks: Vec<T>, hub: TrajectoryHub) -> Self {
        let (progress_tx, progress_store) = new_progress_store();
        Self {
            tasks,
            hub,
            progress_store,
            progress_tx,
            epochs_run: 0,
        }
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
        for (task_index, task) in self.tasks.iter_mut().enumerate() {
            let progress = ProgressHandle::new(task_index, epoch, self.progress_tx.clone());
            let context = TaskContext::new(&self.hub, &progress, epoch);
            task.evolve_one_epoch(&context)?;
        }
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
            epochs_run: _,
        } = self;
        hub.shutdown()?;
        Ok(tasks)
    }
}
