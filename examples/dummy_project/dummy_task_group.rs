use std::io;
use std::path::{Path, PathBuf};

use sci_task_mgr::config_io::{TaskGroupConfig, load_task_config};
use sci_task_mgr::runner::run_tasks_from_configs;
use sci_task_mgr::task::Task;

use super::dummy_task::{TaskConfigList, TaskState};

// Example file names kept next to the example source itself.
const TASK_GROUP_CONFIG_FILE: &str = "task_group_config.toml";
const TASK_CONFIG_FILE: &str = "task_configs.toml";

#[derive(Debug)]
pub struct TaskGroup {
    // Output root selected by the top-level example entrypoint.
    output_dir: PathBuf,
    // Manager-owned task-group config loaded from TOML.
    task_group_config: TaskGroupConfig,
    // One owned task config per task that will be built and run.
    task_configs: Vec<super::dummy_task::TaskConfig>,
}

impl TaskGroup {
    // Load both config scopes from disk and rewrite the task-group output
    // location to the caller-provided example output directory.
    pub fn new(output_dir: impl AsRef<Path>, example_dir: impl AsRef<Path>) -> io::Result<Self> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let example_dir = example_dir.as_ref();
        let task_group_config_path = example_dir.join(TASK_GROUP_CONFIG_FILE);
        let task_config_path = example_dir.join(TASK_CONFIG_FILE);

        let mut task_group_config = TaskGroupConfig::from_path(&task_group_config_path)
            .map_err(|err| io::Error::other(err.to_string()))?;
        task_group_config.io.task_group_dir = output_dir.to_string_lossy().into_owned();

        let task_config_list: TaskConfigList =
            load_task_config(&task_config_path).map_err(|err| io::Error::other(err.to_string()))?;

        Ok(Self {
            output_dir,
            task_group_config,
            task_configs: task_config_list.tasks,
        })
    }

    pub fn run(&mut self) -> io::Result<Vec<PathBuf>> {
        // Hand the full task list to sci_task_mgr and then collect the
        // manager-derived task directories as a compact summary.
        let finished = run_tasks_from_configs::<TaskState>(
            &self.task_group_config,
            self.task_configs.clone(),
        )
        .map_err(|err| io::Error::other(format!("{err:?}")))?;

        let written = finished
            .iter()
            .map(|task| task_output_dir(&self.output_dir, task.config()))
            .collect();
        Ok(written)
    }
}

fn task_output_dir(output_dir: &Path, config: &super::dummy_task::TaskConfig) -> PathBuf {
    // Mirror the current manager-side directory naming format so the example
    // can print the concrete task directories after the run.
    output_dir.join("tasks").join(format!(
        "drift-{}_mission-{}_steps-[{}]",
        config.drift,
        config.mission,
        config
            .steps
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    ))
}
