use sci_task_mgr::config_io::{
    ConfigError, IoConfig, RunConfig, SUPPORTED_SCHEMA_VERSION, TaskGroupConfig, load_task_config,
};
use serde::Deserialize;

/// Verify that the documented TOML envelope parses and validates successfully.
#[test]
fn parses_minimal_toml_envelope() {
    let raw = r#"
schema_version = 1

[run]
name = "screening"
task_type = "dses_screening"
num_threads = 8
num_task_threads = 4

[io]
task_group_dir = "/tmp/out"

[progress]
enabled = true
refresh_hz = 5
"#;

    let config: TaskGroupConfig = toml::from_str(raw).expect("config should parse");
    config.validate().expect("config should validate");

    assert_eq!(config.schema_version, SUPPORTED_SCHEMA_VERSION);
    assert_eq!(config.run.task_type, "dses_screening");
    assert_eq!(config.run.num_threads, 8);
    assert_eq!(config.run.num_task_threads, Some(4));
    assert_eq!(config.io.task_group_dir, "/tmp/out");
    assert_eq!(config.io.task_dir(3), "/tmp/out/tasks/task_0003");
    assert_eq!(config.io.trajectory_dir(3), "/tmp/out/tasks/task_0003/trajectories");
}

/// Verify that validation rejects unsupported schema versions.
#[test]
fn rejects_wrong_schema_version() {
    let config = TaskGroupConfig {
        schema_version: 2,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_threads: 1,
            num_task_threads: None,
            max_epochs: None,
        },
        io: IoConfig {
            task_group_dir: "/tmp/out".to_string(),
        },
        progress: None,
    };

    let err = config.validate().expect_err("schema version should fail");
    assert!(matches!(
        err,
        ConfigError::UnsupportedSchemaVersion {
            found: 2,
            supported: 1
        }
    ));
}

/// Verify that both thread-limit fields reject zero.
#[test]
fn rejects_zero_thread_counts() {
    let config = TaskGroupConfig {
        schema_version: 1,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_threads: 0,
            num_task_threads: Some(0),
            max_epochs: None,
        },
        io: IoConfig {
            task_group_dir: "/tmp/out".to_string(),
        },
        progress: None,
    };

    let err = config.validate().expect_err("thread limits should fail");
    assert!(matches!(err, ConfigError::InvalidField("run.num_threads", _)));
}

/// Verify that validation rejects an empty task directory.
#[test]
fn rejects_empty_task_dir() {
    let config = TaskGroupConfig {
        schema_version: 1,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_threads: 1,
            num_task_threads: None,
            max_epochs: None,
        },
        io: IoConfig {
            task_group_dir: "   ".to_string(),
        },
        progress: None,
    };

    let err = config.validate().expect_err("task dir should fail");
    assert!(matches!(
        err,
        ConfigError::InvalidField("io.task_group_dir", _)
    ));
}

/// Minimal task-owned config used to verify separate task-config loading.
#[derive(Debug, Deserialize, PartialEq)]
struct DemoTaskConfig {
    mission: String,
    steps: Vec<u64>,
}

/// Minimal task-config list wrapper used to verify separate task-config loading.
#[derive(Debug, Deserialize, PartialEq)]
struct DemoTaskConfigList {
    tasks: Vec<DemoTaskConfig>,
}

/// Verify that task-owned config can be loaded independently from the task-group config.
#[test]
fn loads_task_owned_toml_config() {
    let path = "/home/mgr/Projects/sci_task_mgr/examples/dummy_project/task_configs.toml";
    let config: DemoTaskConfigList = load_task_config(path).expect("task config should load");

    assert_eq!(
        config,
        DemoTaskConfigList {
            tasks: vec![
                DemoTaskConfig {
                    mission: "alpha".to_string(),
                    steps: vec![3, 4],
                },
                DemoTaskConfig {
                    mission: "beta".to_string(),
                    steps: vec![5, 8],
                },
                DemoTaskConfig {
                    mission: "gamma".to_string(),
                    steps: vec![13, 21],
                },
            ],
        }
    );
}
