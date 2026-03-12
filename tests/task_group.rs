use std::convert::Infallible;

use sci_task_mgr::task::Task;
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
    type Trajectory = Vec<usize>;
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

    fn evolve_one_epoch(&mut self) -> Result<(), Self::Error> {
        self.epoch += 1;
        self.trajectory.push(self.epoch);
        Ok(())
    }

    fn trajectory(&self) -> &Self::Trajectory {
        &self.trajectory
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
    let mut group = TaskGroup::new(tasks);

    group.run_one_epoch().expect("group should run one epoch");

    assert_eq!(group.epochs_run(), 1);
    assert_eq!(group.tasks()[0].trajectory(), &vec![0, 1]);
    assert_eq!(group.tasks()[1].trajectory(), &vec![3, 4]);
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
    let mut group = TaskGroup::new(tasks);

    group.run_epochs(3).expect("group should run epochs");

    assert_eq!(group.epochs_run(), 3);
    assert_eq!(group.tasks()[0].trajectory(), &vec![2, 3, 4, 5]);
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
    let mut group = TaskGroup::new(tasks);

    assert_eq!(group.len(), 1);
    assert!(!group.is_empty());
    assert_eq!(group.tasks()[0].config().label, "demo");

    group.tasks_mut()[0]
        .evolve_one_epoch()
        .expect("task mutation should work");

    let tasks = group.into_tasks();
    assert_eq!(tasks[0].trajectory(), &vec![1, 2]);
}
