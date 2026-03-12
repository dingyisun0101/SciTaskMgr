use std::convert::Infallible;

use sci_task_io::trajectory::TrajectoryHub;
use sci_task_mgr::progress::{ProgressEventKind, new_progress_store, ProgressHandle};
use sci_task_mgr::task::{Task, TaskContext};
use sci_task_mgr::task_group::TaskGroup;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DummyConfig {
    label: String,
    initial_epoch: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DummyCheckpoint {
    epoch: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DummyTask {
    config: DummyConfig,
    epoch: usize,
    trajectory: Vec<usize>,
}

impl Task for DummyTask {
    type Config = DummyConfig;
    type Checkpoint = DummyCheckpoint;
    type Error = Infallible;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: config.initial_epoch,
            trajectory: vec![config.initial_epoch],
            config,
        })
    }

    fn rebuild_from(
        config: Self::Config,
        checkpoint: Self::Checkpoint,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            config,
            epoch: checkpoint.epoch,
            trajectory: vec![checkpoint.epoch],
        })
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error> {
        let _ = context.hub();
        context.progress().epoch_started();
        self.epoch += 1;
        self.trajectory.push(self.epoch);
        context.progress().epoch_completed();
        Ok(())
    }
}

#[test]
fn runs_one_epoch_across_all_tasks() {
    let tasks = vec![
        DummyTask::new(DummyConfig {
            label: "a".to_string(),
            initial_epoch: 0,
        })
        .expect("task should build"),
        DummyTask::new(DummyConfig {
            label: "b".to_string(),
            initial_epoch: 3,
        })
        .expect("task should build"),
    ];
    let hub = TrajectoryHub::start(1).expect("hub should start");
    let mut group = TaskGroup::new(tasks, hub);

    group.run_one_epoch().expect("group should run one epoch");
    group.drain_progress();

    assert_eq!(group.epochs_run(), 1);
    assert_eq!(group.tasks()[0].trajectory, vec![0, 1]);
    assert_eq!(group.tasks()[1].trajectory, vec![3, 4]);
    assert_eq!(group.progress_events().len(), 4);
    group.shutdown().expect("group should shut down");
}

#[test]
fn runs_multiple_epochs() {
    let tasks = vec![
        DummyTask::new(DummyConfig {
            label: "demo".to_string(),
            initial_epoch: 2,
        })
        .expect("task should build"),
    ];
    let hub = TrajectoryHub::start(1).expect("hub should start");
    let mut group = TaskGroup::new(tasks, hub);

    group.run_epochs(3).expect("group should run epochs");
    group.drain_progress();

    assert_eq!(group.epochs_run(), 3);
    assert_eq!(group.tasks()[0].trajectory, vec![2, 3, 4, 5]);
    assert_eq!(group.progress_events().len(), 6);
    group.shutdown().expect("group should shut down");
}

#[test]
fn exposes_task_accessors() {
    let tasks = vec![
        DummyTask::new(DummyConfig {
            label: "demo".to_string(),
            initial_epoch: 1,
        })
        .expect("task should build"),
    ];
    let hub = TrajectoryHub::start(1).expect("hub should start");
    let mut group = TaskGroup::new(tasks, hub);

    assert_eq!(group.len(), 1);
    assert!(!group.is_empty());
    assert_eq!(group.tasks()[0].config().label, "demo");

    let hub = group.hub().clone();
    let (tx, mut store) = new_progress_store();
    let progress = ProgressHandle::new(0, 2, tx);
    let context = TaskContext::new(&hub, &progress, 2);
    group.tasks_mut()[0]
        .evolve_one_epoch(&context)
        .expect("task mutation should work");
    store.drain();

    let tasks = group.shutdown().expect("group should shut down");
    assert_eq!(tasks[0].trajectory, vec![1, 2]);
    assert_eq!(store.snapshot()[0].kind, ProgressEventKind::EpochStarted);
}
