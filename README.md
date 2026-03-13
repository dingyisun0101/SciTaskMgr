# sci_task_mgr

`sci_task_mgr` is a Rust crate for running scientific tasks in epoch-sized units.

The current design is built around four ideas:

- manager-owned config envelope parsing
- task-owned scientific config and state
- epoch-based task-group orchestration
- manager-owned runtime services passed into tasks through `TaskContext`

Trajectory persistence is delegated to [`sci_task_io`](https://github.com/dingyisun0101/SciTaskIO). A `TaskGroup` owns a shared `TrajectoryHub`, and each task is expected to submit its epoch trajectory to that hub during `evolve_one_epoch(...)`.

## Status

This crate is still early-stage. The implemented surface today includes:

- `config_io`
  - TOML-only task-group config and task-config loading
  - schema version validation
  - task-group-only envelope: `run`, `io`, and optional `progress`
- `runner`
  - manager-owned task construction and execution from `TaskGroupConfig`
- `task`
  - `Task` trait
  - config-owned task construction helpers
  - `TaskContext` with shared runtime services
- `task_group`
  - generic `TaskGroup<T>`
  - epoch-by-epoch execution across all tasks
  - shared `TrajectoryHub` ownership
- `progress`
  - internal event-based progress handles and in-memory event store

## Install

```toml
[dependencies]
sci_task_mgr = "0.0.1"
```

## Config

Task-group config is TOML-only.

Examples:

- [`examples/task_group_config.toml`](/home/mgr/Projects/sci_task_mgr/examples/task_group_config.toml)
- [`examples/task_config.toml`](/home/mgr/Projects/sci_task_mgr/examples/task_config.toml)

Current task-group-owned envelope:

- `schema_version`
- `run`
- `io`
- `progress`

Task-owned scientific config lives separately, either as a TOML file loaded with `load_task_config(...)` or as programmatic `Vec<T::Config>` values.

The `io.task_group_dir` field is the only required manager-owned path. Per-task directories are derived automatically under `task_group_dir/tasks/task_XXXX`.

## Core API

Main modules:

- `sci_task_mgr::config_io`
- `sci_task_mgr::runner`
- `sci_task_mgr::task`
- `sci_task_mgr::task_group`
- `sci_task_mgr::progress`

Important types:

- `ManagerConfig`
- `TaskGroupConfig`
- `Task`
- `TaskContext`
- `TaskRunnerError`
- `TaskGroup`
- `ProgressHandle`
- `ProgressStore`

## Task Model

Concrete tasks implement `Task` and own their config directly.

The current contract is:

- `new(config)`
- `config()`
- `evolve_one_epoch(&mut self, context)`

`TaskContext` currently provides:

- shared `TrajectoryHub` access
- progress reporting handle
- manager-assigned task index
- manager-assigned epoch number
- task-scoped Rayon compute pool access
- task-scoped `num_threads` limit for internal parallel work
- manager-owned trajectory directory
- manager-owned trajectory submission with derived paths

Typical end-user entry point:

```rust
use sci_task_mgr::runner::run_tasks_from_configs;

let finished_tasks = run_tasks_from_configs::<MyTask>(&task_group_config, task_configs)?;
```

## Task Groups

`TaskGroup<T>` is a generic epoch runner over `Vec<T>`.

It currently supports:

- constructing a group from manager-owned runtime config
- adding tasks after construction
- configuring per-task compute threads and task-group parallelism
- running one epoch across all tasks
- running multiple epochs
- draining and inspecting collected progress events
- shutting down the shared hub

## Progress

Progress reporting is internal and event-based.

Tasks do not own logger state directly. Instead, a `TaskGroup` creates task-scoped `ProgressHandle`s each epoch and passes them through `TaskContext`.

Available event kinds today:

- `EpochStarted`
- `EpochCompleted`
- `Message(String)`

## Development

Run tests:

```bash
cargo test
```

## Notes

- Config format is `TOML` only.
- Trajectory IO is delegated to `sci_task_io`.
- The API is expected to evolve as task-group and trajectory-hand-off behavior becomes stricter.
