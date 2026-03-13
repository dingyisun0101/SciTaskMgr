use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use rayon::current_num_threads;
use serde::Serialize;
use sci_task_mgr::progress::{ProgressEventKind, new_progress_store, ProgressHandle};
use sci_task_mgr::task::{Task, TaskContext};
use sci_task_mgr::task_group::{TaskGroup, TaskGroupRuntimeConfig};

/// Minimal config used by the task-group runner tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DummyConfig {
    label: String,
    initial_epoch: usize,
}

/// Basic task fixture that records epochs and observed compute-pool width.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DummyTask {
    config: DummyConfig,
    epoch: usize,
    trajectory: Vec<usize>,
    compute_threads_seen: usize,
}

impl Task for DummyTask {
    type Config = DummyConfig;
    type Error = Infallible;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: config.initial_epoch,
            trajectory: vec![config.initial_epoch],
            compute_threads_seen: 0,
            config,
        })
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error> {
        let _ = context.hub();
        context.progress().epoch_started();
        self.compute_threads_seen = context.install_compute_pool(current_num_threads);
        assert_eq!(self.compute_threads_seen, context.num_threads());
        self.epoch += 1;
        self.trajectory.push(self.epoch);
        context.progress().epoch_completed();
        Ok(())
    }
}

/// Verify that one group epoch advances every task once.
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
    let mut group = TaskGroup::new(TaskGroupRuntimeConfig {
        task_group_dir: std::env::temp_dir().join("sci_task_mgr_group_one_epoch"),
        task_num_threads: 2,
        num_task_threads: None,
    })
    .expect("group should build");
    group.add_tasks(tasks).expect("tasks should be added");

    group.run_one_epoch().expect("group should run one epoch");
    group.drain_progress();

    assert_eq!(group.epochs_run(), 1);
    assert_eq!(group.task_num_threads(), 2);
    assert_eq!(group.num_task_threads(), None);
    assert_eq!(group.tasks()[0].trajectory, vec![0, 1]);
    assert_eq!(group.tasks()[1].trajectory, vec![3, 4]);
    assert_eq!(group.tasks()[0].compute_threads_seen, 2);
    assert_eq!(group.tasks()[1].compute_threads_seen, 2);
    assert_eq!(group.progress_events().len(), 4);
    group.shutdown().expect("group should shut down");
}

/// Verify that repeated group epochs preserve task-local state evolution.
#[test]
fn runs_multiple_epochs() {
    let tasks = vec![
        DummyTask::new(DummyConfig {
            label: "demo".to_string(),
            initial_epoch: 2,
        })
        .expect("task should build"),
    ];
    let mut group = TaskGroup::new(TaskGroupRuntimeConfig {
        task_group_dir: std::env::temp_dir().join("sci_task_mgr_group_many_epochs"),
        task_num_threads: 3,
        num_task_threads: Some(1),
    })
    .expect("group should build");
    group.add_tasks(tasks).expect("tasks should be added");

    group.run_epochs(3).expect("group should run epochs");
    group.drain_progress();

    assert_eq!(group.epochs_run(), 3);
    assert_eq!(group.tasks()[0].trajectory, vec![2, 3, 4, 5]);
    assert_eq!(group.progress_events().len(), 6);
    group.shutdown().expect("group should shut down");
}

/// Verify the basic task-group accessors and task mutation path.
#[test]
fn exposes_task_accessors() {
    let tasks = vec![
        DummyTask::new(DummyConfig {
            label: "demo".to_string(),
            initial_epoch: 1,
        })
        .expect("task should build"),
    ];
    let mut group = TaskGroup::new(TaskGroupRuntimeConfig {
        task_group_dir: std::env::temp_dir().join("sci_task_mgr_group_accessors"),
        task_num_threads: 1,
        num_task_threads: Some(1),
    })
    .expect("group should build");
    group.add_tasks(tasks).expect("tasks should be added");

    assert_eq!(group.len(), 1);
    assert!(!group.is_empty());
    assert_eq!(group.tasks()[0].config().label, "demo");

    let hub = group.hub().clone();
    let (tx, mut store) = new_progress_store();
    let progress = ProgressHandle::new(0, 2, tx);
    let compute_pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .expect("pool should build"),
    );
    let context = TaskContext::new(
        &hub,
        &progress,
        0,
        2,
        1,
        std::env::temp_dir().join("sci_task_mgr_group_accessors_task"),
        Arc::new(std::sync::atomic::AtomicU64::new(0)),
        compute_pool,
    );
    group.tasks_mut()[0]
        .evolve_one_epoch(&context)
        .expect("task mutation should work");
    store.drain();

    let tasks = group.shutdown().expect("group should shut down");
    assert_eq!(tasks[0].trajectory, vec![1, 2]);
    assert_eq!(store.snapshot()[0].kind, ProgressEventKind::EpochStarted);
}

/// Blocking task fixture used to measure group-level concurrency.
#[derive(Debug)]
struct ConcurrencyTask {
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
}

impl Task for ConcurrencyTask {
    type Config = ();
    type Error = Infallible;

    fn new(_config: Self::Config) -> Result<Self, Self::Error> {
        unreachable!("not used in test")
    }

    fn config(&self) -> &Self::Config {
        static CONFIG: () = ();
        &CONFIG
    }

    fn evolve_one_epoch(&mut self, _context: &TaskContext<'_>) -> Result<(), Self::Error> {
        let active_now = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_active.fetch_max(active_now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(40));
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Verify that the task-group concurrency cap limits simultaneous task execution.
#[test]
fn caps_how_many_tasks_run_in_parallel() {
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let tasks: Vec<_> = (0..4)
        .map(|_| ConcurrencyTask {
            active: Arc::clone(&active),
            max_active: Arc::clone(&max_active),
        })
        .collect();
    let mut group = TaskGroup::new(TaskGroupRuntimeConfig {
        task_group_dir: std::env::temp_dir().join("sci_task_mgr_group_concurrency"),
        task_num_threads: 1,
        num_task_threads: Some(2),
    })
    .expect("group should build");
    group.add_tasks(tasks).expect("tasks should be added");

    group.run_one_epoch().expect("group should run");
    assert_eq!(max_active.load(Ordering::SeqCst), 2);
    group.shutdown().expect("group should shut down");
}
