use std::convert::Infallible;

use sci_task_io::trajectory::TrajectoryHub;
use sci_task_mgr::progress::{ProgressEventKind, new_progress_store, ProgressHandle};
use sci_task_mgr::task::{Task, TaskContext, build_task, build_task_copies, build_tasks_from_configs};

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
        context.progress().message(format!("epoch {} complete", self.epoch));
        context.progress().epoch_completed();
        Ok(())
    }
}

#[test]
fn builds_single_task_from_owned_config() {
    let config = DummyConfig {
        label: "demo".to_string(),
        initial_epoch: 3,
    };

    let task = build_task::<DummyTask>(config).expect("task should build");

    assert_eq!(task.config().label, "demo");
    assert_eq!(task.epoch, 3);
    assert_eq!(task.trajectory, vec![3]);
}

#[test]
fn clones_config_when_building_multiple_tasks() {
    let config = DummyConfig {
        label: "demo".to_string(),
        initial_epoch: 1,
    };

    let tasks = build_task_copies::<DummyTask>(config, 3).expect("tasks should build");

    assert_eq!(tasks.len(), 3);
    assert!(tasks.iter().all(|task| task.config().label == "demo"));
    assert!(tasks.iter().all(|task| task.epoch == 1));
    assert!(tasks.iter().all(|task| task.trajectory == vec![1]));
}

#[test]
fn builds_tasks_from_distinct_configs() {
    let configs = vec![
        DummyConfig {
            label: "a".to_string(),
            initial_epoch: 0,
        },
        DummyConfig {
            label: "b".to_string(),
            initial_epoch: 5,
        },
    ];

    let tasks = build_tasks_from_configs::<DummyTask>(configs).expect("tasks should build");

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].config().label, "a");
    assert_eq!(tasks[1].config().label, "b");
    assert_eq!(tasks[1].epoch, 5);
    assert_eq!(tasks[1].trajectory, vec![5]);
}

#[test]
fn can_rebuild_task_from_checkpoint() {
    let config = DummyConfig {
        label: "demo".to_string(),
        initial_epoch: 0,
    };
    let checkpoint = DummyCheckpoint { epoch: 7 };

    let task = DummyTask::rebuild_from(config, checkpoint).expect("task should rebuild");

    assert_eq!(task.epoch, 7);
    assert_eq!(task.trajectory, vec![7]);
}

#[test]
fn evolves_with_task_context() {
    let hub = TrajectoryHub::start(1).expect("hub should start");
    let mut task = DummyTask::new(DummyConfig {
        label: "demo".to_string(),
        initial_epoch: 2,
    })
    .expect("task should build");
    let (tx, mut store) = new_progress_store();
    let progress = ProgressHandle::new(0, 3, tx);
    let context = TaskContext::new(&hub, &progress, 3);

    task.evolve_one_epoch(&context).expect("task should evolve");
    store.drain();

    assert_eq!(task.epoch, 3);
    assert_eq!(task.trajectory, vec![2, 3]);
    assert_eq!(store.snapshot().len(), 3);
    assert_eq!(store.snapshot()[0].kind, ProgressEventKind::EpochStarted);
    hub.shutdown().expect("hub should shut down");
}
