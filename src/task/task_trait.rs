use super::TaskContext;

/// Core contract for a scientific task managed by `sci_task_mgr`.
pub trait Task: Sized + Send + 'static {
    /// Owned config type stored inside the task itself.
    type Config: Clone + Send + Sync + 'static;
    /// Task-specific checkpoint type used for rebuild.
    type Checkpoint: Send + 'static;
    /// Error type returned by task construction and execution.
    type Error: Send;

    /// Construct a fresh task from owned config.
    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    /// Reconstruct a task from owned config plus a checkpoint value.
    fn rebuild_from(
        config: Self::Config,
        checkpoint: Self::Checkpoint,
    ) -> Result<Self, Self::Error>;

    /// Return the owned config currently associated with the task.
    fn config(&self) -> &Self::Config;

    /// Advance the task by one epoch using manager-owned runtime services.
    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error>;
}
