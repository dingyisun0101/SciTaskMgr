/// Config loading and validation for the manager-owned config envelope.
pub mod config_io;
/// Internal event-based progress reporting utilities.
pub mod progress;
/// Manager-owned task runner helpers that hide task-group orchestration.
pub mod runner;
/// Core task traits and task execution context.
pub mod task;
/// Generic epoch-based orchestration over a collection of tasks.
pub mod task_group;
