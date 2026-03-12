use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use sci_task_io::trajectory::{Trajectory, TrajectoryHub};
use sci_task_mgr::task::{Task, TaskContext};
use sci_task_mgr::task_group::TaskGroup;

#[derive(Debug, Clone)]
struct SubmissionConfig {
    output_path: PathBuf,
    initial_epoch: u64,
}

#[derive(Debug, Clone)]
struct SubmissionCheckpoint {
    epoch: u64,
}

#[derive(Debug)]
struct SubmissionTask {
    config: SubmissionConfig,
    epoch: u64,
}

impl Task for SubmissionTask {
    type Config = SubmissionConfig;
    type Checkpoint = SubmissionCheckpoint;
    type Error = std::io::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: config.initial_epoch,
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
        })
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error> {
        context.progress().epoch_started();
        self.epoch += 1;

        let mut trajectory = Trajectory::new();
        trajectory
            .metadata_mut()
            .insert("epoch".to_string(), serde_json::Value::from(self.epoch));
        trajectory
            .push_row("state", self.epoch, vec![self.epoch as f64])
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        let write_handle =
            context.submit_trajectory_tracked(trajectory, self.config.output_path.clone())?;
        write_handle.wait()?;
        context.progress().epoch_completed();
        Ok(())
    }
}

fn unique_output_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be available")
        .as_nanos();
    std::env::temp_dir().join(format!("sci_task_mgr_epoch_submission_{nonce}.json"))
}

#[test]
fn task_submits_epoch_trajectory_through_context_hub() {
    let output_path = unique_output_path();
    let config = SubmissionConfig {
        output_path: output_path.clone(),
        initial_epoch: 0,
    };
    let task = SubmissionTask::new(config).expect("task should build");
    let hub = TrajectoryHub::start(1).expect("hub should start");
    let mut group = TaskGroup::new(vec![task], hub);

    group.run_one_epoch().expect("group should run");
    group.drain_progress();
    group.shutdown().expect("group should shut down");

    let loaded =
        Trajectory::from_json(&output_path, false).expect("trajectory written by hub should load");
    let state = loaded
        .track_by_label("state")
        .expect("state track should exist");
    let (time, signal) = state.latest_row().expect("row should exist");

    assert_eq!(time, &serde_json::Value::from(1_u64));
    assert_eq!(signal, &serde_json::json!([1.0]));
    assert_eq!(loaded.metadata()["epoch"], serde_json::Value::from(1_u64));

    std::fs::remove_file(output_path).expect("temp trajectory should be removed");
}
