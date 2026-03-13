use std::io;

use sci_task_io::trajectory::Trajectory;
use sci_task_mgr::task::{Task, TaskContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Every dummy task records one evolving state vector under a single label.
const STATE_LABEL: &str = "state";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskConfig {
    // Human-readable task name. This participates in the derived task
    // directory name managed by sci_task_mgr.
    pub mission: String,
    // One small task-specific parameter sweep vector.
    pub steps: Vec<u64>,
    // Constant drift added to the dummy state each epoch.
    pub drift: f64,
}

#[derive(Debug)]
pub struct TaskState {
    // Owned task config kept for inspection and manager-derived directory
    // naming.
    config: TaskConfig,
    // Logical epoch count completed by this task so far.
    epoch: u64,
    // Tiny state vector that changes deterministically each epoch.
    values: Vec<f64>,
}

impl Task for TaskState {
    type Config = TaskConfig;
    type Error = io::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        let values = config
            .steps
            .iter()
            .map(|step| (*step as f64) * config.drift)
            .collect();
        Ok(Self {
            config,
            epoch: 0,
            values,
        })
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn evolve_one_epoch(&mut self, context: &TaskContext<'_>) -> Result<(), Self::Error> {
        context.progress().epoch_started();
        self.epoch += 1;

        // Run a small in-task parallel update through the task-scoped Rayon
        // pool managed by sci_task_mgr.
        let drift = self.config.drift;
        let epoch = self.epoch;
        let updated = context.install_compute_pool(|| {
            self.values
                .iter()
                .enumerate()
                .map(|(idx, value)| value + drift + (idx as f64) + (epoch as f64))
                .collect::<Vec<_>>()
        });
        self.values = updated.clone();

        let mut trajectory = Trajectory::new();
        trajectory
            .metadata_mut()
            .insert("mission".to_string(), Value::from(self.config.mission.clone()));
        trajectory
            .metadata_mut()
            .insert("epoch".to_string(), Value::from(self.epoch));
        trajectory
            .metadata_mut()
            .insert("task_index".to_string(), Value::from(context.task_index() as u64));
        trajectory
            .push_row(STATE_LABEL, self.epoch, updated)
            .map_err(|err| io::Error::other(err.to_string()))?;

        let write_handle = context.submit_trajectory_tracked(STATE_LABEL, trajectory)?;
        write_handle.wait()?;
        context.progress().epoch_completed();
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskConfigList {
    // One owned config per task in the task group.
    pub tasks: Vec<TaskConfig>,
}
