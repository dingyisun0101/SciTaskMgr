use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sci_task_io::trajectory::Trajectory;
use sci_task_mgr::config_io::{IoConfig, RunConfig, TaskGroupConfig};
use sci_task_mgr::runner::run_tasks_from_configs;
use sci_task_mgr::task::{Task, TaskContext};

/// Config fixture for a task that writes one trajectory file per epoch.
#[derive(Debug, Clone, Serialize)]
struct SubmissionConfig {
    initial_epoch: u64,
}

/// Task fixture that submits a tracked trajectory write during each epoch.
#[derive(Debug)]
struct SubmissionTask {
    config: SubmissionConfig,
    epoch: u64,
}

impl Task for SubmissionTask {
    type Config = SubmissionConfig;
    type Error = std::io::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: config.initial_epoch,
            config,
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

        let write_handle = context.submit_trajectory_tracked("state", trajectory)?;
        write_handle.wait()?;
        context.progress().epoch_completed();
        Ok(())
    }
}

/// Generate a collision-resistant temporary output directory for trajectory tests.
fn unique_output_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be available")
        .as_nanos();
    std::env::temp_dir().join(format!("sci_task_mgr_epoch_submission_{nonce}"))
}

/// Verify that trajectories submitted through `TaskContext` reach the shared hub.
#[test]
fn task_submits_epoch_trajectory_through_context_hub() {
    let output_dir = unique_output_dir();
    let config = SubmissionConfig { initial_epoch: 0 };
    let task_group_config = TaskGroupConfig {
        schema_version: 1,
        run: RunConfig {
            name: "submission".to_string(),
            task_type: "submission".to_string(),
            num_threads: 1,
            num_task_threads: Some(1),
            max_epochs: Some(1),
        },
        io: IoConfig {
            task_group_dir: output_dir.to_string_lossy().into_owned(),
        },
        progress: None,
    };

    let tasks = run_tasks_from_configs::<SubmissionTask>(&task_group_config, vec![config])
        .expect("run should succeed");
    assert_eq!(tasks[0].epoch, 1);

    let output_path = output_dir
        .join("tasks")
        .join("initial_epoch-0")
        .join("trajectories")
        .join("epoch_1_state_0.json");
    let loaded =
        Trajectory::from_json(&output_path, false).expect("trajectory written by hub should load");
    let state = loaded
        .track_by_label("state")
        .expect("state track should exist");
    let (time, signal) = state.latest_row().expect("row should exist");

    assert_eq!(time, &serde_json::Value::from(1_u64));
    assert_eq!(signal, &serde_json::json!([1.0]));
    assert_eq!(loaded.metadata()["epoch"], serde_json::Value::from(1_u64));

    std::fs::remove_dir_all(output_dir).expect("temp trajectory tree should be removed");
}
