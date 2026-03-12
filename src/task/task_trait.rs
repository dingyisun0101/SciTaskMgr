pub trait Task: Sized + Send + 'static {
    type Config: Clone + Send + Sync + 'static;
    type Checkpoint: Send + 'static;
    type Trajectory;
    type Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error>;

    fn rebuild_from(
        config: Self::Config,
        checkpoint: Self::Checkpoint,
    ) -> Result<Self, Self::Error>;

    fn config(&self) -> &Self::Config;

    fn evolve_one_epoch(&mut self) -> Result<(), Self::Error>;

    fn trajectory(&self) -> &Self::Trajectory;
}
