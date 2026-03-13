use sci_task_mgr::config_io::{
    CheckpointConfig, ConfigError, IoConfig, ManagerConfig, ResumePolicy, RunConfig,
    SUPPORTED_SCHEMA_VERSION,
};
use toml::Value;

/// Verify that the documented TOML envelope parses and validates successfully.
#[test]
fn parses_minimal_toml_envelope() {
    let raw = r#"
schema_version = 1

[run]
name = "screening"
task_type = "dses_screening"
num_threads = 8
num_tasks = 50
num_task_threads = 4

[io]
root_dir = "/tmp/out"

[checkpoint]
resume = "if_available"
cleanup_invalid = true
sync_group_epochs = false

[progress]
enabled = true
refresh_hz = 5

[task]
mission = "b2"
steps = [3, 4]
"#;

    let config: ManagerConfig = toml::from_str(raw).expect("config should parse");
    config.validate().expect("config should validate");

    assert_eq!(config.schema_version, SUPPORTED_SCHEMA_VERSION);
    assert_eq!(config.run.task_type, "dses_screening");
    assert_eq!(config.run.num_threads, 8);
    assert_eq!(config.run.num_task_threads, Some(4));
    assert_eq!(config.task["mission"], Value::String("b2".to_string()));
}

/// Verify that validation rejects unsupported schema versions.
#[test]
fn rejects_wrong_schema_version() {
    let config = ManagerConfig {
        schema_version: 2,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_threads: 1,
            num_tasks: None,
            num_task_threads: None,
            max_epochs: None,
        },
        io: IoConfig {
            root_dir: "/tmp/out".to_string(),
            trajectory_dir: None,
            checkpoint_dir: None,
        },
        checkpoint: CheckpointConfig {
            resume: ResumePolicy::IfAvailable,
            cleanup_invalid: false,
            sync_group_epochs: false,
        },
        progress: None,
        task: Value::String("task_payload".to_string()),
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
    let config = ManagerConfig {
        schema_version: 1,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_threads: 0,
            num_tasks: None,
            num_task_threads: Some(0),
            max_epochs: None,
        },
        io: IoConfig {
            root_dir: "/tmp/out".to_string(),
            trajectory_dir: None,
            checkpoint_dir: None,
        },
        checkpoint: CheckpointConfig {
            resume: ResumePolicy::IfAvailable,
            cleanup_invalid: false,
            sync_group_epochs: false,
        },
        progress: None,
        task: Value::String("task_payload".to_string()),
    };

    let err = config.validate().expect_err("thread limits should fail");
    assert!(matches!(err, ConfigError::InvalidField("run.num_threads", _)));
}
