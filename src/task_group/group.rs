use std::io;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use serde::Serialize;
use serde_json::Value;
use sci_task_io::trajectory::TrajectoryHub;

use crate::progress::{ProgressEvent, ProgressStore, new_progress_store, ProgressHandle};
use crate::task::{Task, TaskContext};

/// Manager-owned runtime settings for one task group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGroupRuntimeConfig {
    /// Base directory for the managed task group and its derived task directories.
    pub task_group_dir: PathBuf,
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
    config: TaskGroupRuntimeConfig,
    num_writer_threads: usize,
    group_pool: ThreadPool,
    task_pools: Vec<Arc<ThreadPool>>,
    task_dirs: Vec<PathBuf>,
    epochs_run: u64,
}

impl<T: Task> TaskGroup<T> {
    /// Create a new task group from manager-owned runtime configuration.
    pub fn new(config: TaskGroupRuntimeConfig) -> Result<Self, TaskGroupInitError> {
        let (progress_tx, progress_store) = new_progress_store();
        let num_writer_threads = default_num_writer_threads();
        let hub = TrajectoryHub::start(num_writer_threads)?;
        let group_pool = Self::build_group_pool(config.num_task_threads, 0)?;
        Ok(Self {
            tasks: Vec::new(),
            hub,
            progress_store,
            progress_tx,
            config,
            num_writer_threads,
            group_pool,
            task_pools: Vec::new(),
            task_dirs: Vec::new(),
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
            self.task_dirs
                .push(derive_task_dir(&self.config.task_group_dir, task.config(), task_index, &self.task_dirs));
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
        self.num_writer_threads
    }

    /// Return the base task directory owned by the group.
    pub fn task_dir(&self) -> &std::path::Path {
        &self.config.task_group_dir
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
        let task_dirs = &self.task_dirs;
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
                        task_index,
                        epoch,
                        task_num_threads,
                        task_dirs[task_index].join("trajectories"),
                        Arc::new(AtomicU64::new(0)),
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
            num_writer_threads: _,
            group_pool: _,
            task_pools: _,
            task_dirs: _,
            epochs_run: _,
        } = self;
        hub.shutdown()?;
        Ok(tasks)
    }
}

/// Return the default number of writer threads for the internal trajectory hub.
fn default_num_writer_threads() -> usize {
    std::thread::available_parallelism()
        .map(|parallelism| parallelism.get().min(4))
        .unwrap_or(1)
}

/// Return the task-scoped directory derived from the serialized task config.
fn derive_task_dir<C>(
    task_group_dir: &std::path::Path,
    config: &C,
    task_index: usize,
    existing_task_dirs: &[PathBuf],
) -> PathBuf
where
    C: Serialize,
{
    let stem = config_to_dir_stem(config).unwrap_or_else(|| format!("task-{task_index:04}"));
    let tasks_dir = task_group_dir.join("tasks");
    let mut candidate = tasks_dir.join(&stem);
    let used: HashSet<&std::path::Path> = existing_task_dirs.iter().map(PathBuf::as_path).collect();
    if used.contains(candidate.as_path()) {
        candidate = tasks_dir.join(format!("{stem}_task-{task_index:04}"));
    }
    candidate
}

/// Convert one serialized task config into a deterministic directory stem.
fn config_to_dir_stem<C>(config: &C) -> Option<String>
where
    C: Serialize,
{
    let value = serde_json::to_value(config).ok()?;
    let mut fields = Vec::new();
    flatten_value_fields(None, &value, &mut fields);
    if fields.is_empty() {
        None
    } else {
        Some(fields.join("_"))
    }
}

/// Flatten one serialized config value into `field-value` path fragments.
fn flatten_value_fields(prefix: Option<String>, value: &Value, fields: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                let next_prefix = match &prefix {
                    Some(prefix) => format!("{prefix}-{key}"),
                    None => key.clone(),
                };
                flatten_value_fields(Some(next_prefix), &map[&key], fields);
            }
        }
        _ => {
            if let Some(prefix) = prefix {
                fields.push(format!("{prefix}-{}", format_value(value)));
            }
        }
    }
}

/// Format one scalar or array config value for inclusion in a task directory name.
fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => sanitize_component(string),
        Value::Array(array) => format!(
            "[{}]",
            array
                .iter()
                .map(format_value)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(_) => "object".to_string(),
    }
}

/// Sanitize one directory-name component while preserving the requested separators.
fn sanitize_component(raw: &str) -> String {
    let sanitized: String = raw
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '[' | ']' | ',' | '.' => ch,
            _ => '_',
        })
        .collect();
    if sanitized.is_empty() {
        "value".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{config_to_dir_stem, derive_task_dir};

    #[derive(Serialize)]
    struct DemoConfig {
        mission: &'static str,
        steps: Vec<u64>,
        parameters: Parameters,
    }

    #[derive(Serialize)]
    struct Parameters {
        k: u64,
        dt: f64,
    }

    #[test]
    fn formats_config_values_into_directory_stem() {
        let config = DemoConfig {
            mission: "batch-2",
            steps: vec![3, 4],
            parameters: Parameters { k: 100, dt: 0.1 },
        };

        let stem = config_to_dir_stem(&config).expect("stem should be generated");

        assert_eq!(
            stem,
            "mission-batch-2_parameters-dt-0.1_parameters-k-100_steps-[3,4]"
        );
    }

    #[test]
    fn appends_task_index_when_config_dirs_collide() {
        let config = DemoConfig {
            mission: "batch-2",
            steps: vec![3, 4],
            parameters: Parameters { k: 100, dt: 0.1 },
        };
        let task_group_dir = std::path::Path::new("/tmp/demo-group");
        let first = derive_task_dir(task_group_dir, &config, 0, &[]);
        let second = derive_task_dir(task_group_dir, &config, 1, std::slice::from_ref(&first));

        assert_eq!(
            first,
            std::path::Path::new("/tmp/demo-group/tasks/mission-batch-2_parameters-dt-0.1_parameters-k-100_steps-[3,4]")
        );
        assert_eq!(
            second,
            std::path::Path::new("/tmp/demo-group/tasks/mission-batch-2_parameters-dt-0.1_parameters-k-100_steps-[3,4]_task-0001")
        );
    }
}
