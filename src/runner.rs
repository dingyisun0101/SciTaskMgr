use std::io;

use crate::config_io::TaskGroupConfig;
use crate::task::{Task, build_tasks_from_configs};
use crate::task_group::{TaskGroup, TaskGroupInitError, TaskGroupRuntimeConfig};

/// Error returned while building or executing a manager-owned task run.
#[derive(Debug)]
pub enum TaskRunnerError<E> {
    /// The manager config does not match the requested task construction mode.
    InvalidConfig(&'static str, String),
    /// Building task instances failed.
    Task(E),
    /// Building the underlying task group failed.
    Group(TaskGroupInitError),
    /// Shutting down the task group failed.
    Io(io::Error),
}

/// Build one task per config and execute the configured epochs.
pub fn run_tasks_from_configs<T>(
    task_group_config: &TaskGroupConfig,
    task_configs: Vec<T::Config>,
) -> Result<Vec<T>, TaskRunnerError<T::Error>>
where
    T: Task,
{
    let tasks = build_tasks_from_configs::<T>(task_configs).map_err(TaskRunnerError::Task)?;
    run_tasks(task_group_config, tasks)
}

/// Execute already-built tasks with manager-owned orchestration.
pub fn run_tasks<T>(
    task_group_config: &TaskGroupConfig,
    tasks: Vec<T>,
) -> Result<Vec<T>, TaskRunnerError<T::Error>>
where
    T: Task,
{
    if tasks.is_empty() {
        return Err(TaskRunnerError::InvalidConfig(
            "tasks",
            "at least one task config is required".to_string(),
        ));
    }

    let mut group = TaskGroup::new(TaskGroupRuntimeConfig {
        task_group_dir: task_group_config.io.task_group_dir.clone().into(),
        task_num_threads: task_group_config.run.num_threads,
        num_task_threads: task_group_config.run.num_task_threads,
    })
    .map_err(TaskRunnerError::Group)?;
    group.add_tasks(tasks).map_err(TaskRunnerError::Group)?;

    if let Some(max_epochs) = task_group_config.run.max_epochs {
        group.run_epochs(max_epochs).map_err(TaskRunnerError::Task)?;
    } else {
        group.run_one_epoch().map_err(TaskRunnerError::Task)?;
    }

    group.shutdown().map_err(TaskRunnerError::Io)
}
