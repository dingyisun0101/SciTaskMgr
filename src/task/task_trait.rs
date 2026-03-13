use serde::Serialize;

use super::TaskContext;

/// Core contract for a scientific task managed by `sci_task_mgr`.
pub trait Task: Sized + Send + 'static {
    /// Owned config type stored inside the task itself.
    type Config: Clone + Send + Sync + Serialize + 'static;
    /// Error type returned by task construction and execution.
    type Error: Send;

    /// Construct a fresh task from owned config.
    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    /// Return the owned config currently associated with the task.
    fn config(&self) -> &Self::Config;

    /// Advance the task by one epoch using manager-owned runtime services.
    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error>;
}
