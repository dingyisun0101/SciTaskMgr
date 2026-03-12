mod build;
mod context;
mod task_trait;

/// Helper for constructing a single task from owned config.
pub use build::{build_task, build_task_copies, build_tasks_from_configs};
/// Manager-owned execution context passed into a task for one epoch.
pub use context::TaskContext;
/// Core task contract implemented by concrete scientific tasks.
pub use task_trait::Task;
