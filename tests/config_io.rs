use sci_task_mgr::config_io::{
    CheckpointConfig, ConfigError, IoConfig, ManagerConfig, ResumePolicy, RunConfig,
    SUPPORTED_SCHEMA_VERSION,
};
use toml::Value;

#[test]
fn parses_minimal_toml_envelope() {
    let raw = r#"
schema_version = 1

[run]
name = "screening"
task_type = "dses_screening"
num_tasks = 50

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
    assert_eq!(config.task["mission"], Value::String("b2".to_string()));
}

#[test]
fn rejects_wrong_schema_version() {
    let config = ManagerConfig {
        schema_version: 2,
        run: RunConfig {
            name: "run".to_string(),
            task_type: "demo".to_string(),
            num_tasks: None,
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
