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
  - TOML-only manager config loading
  - schema version validation
  - minimal universal envelope: `run`, `io`, `checkpoint`, optional `progress`, and opaque `task`
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

Manager config is TOML-only.

Example: [`examples/config.toml`](/home/mgr/Projects/sci_task_mgr/examples/config.toml)

Current manager-owned envelope:

- `schema_version`
- `run`
- `io`
- `checkpoint`
- `progress`
- `task`

The `task` section is intentionally left opaque to the manager so end users and concrete task implementations can define most of the scientific payload themselves.

## Core API

Main modules:

- `sci_task_mgr::config_io`
- `sci_task_mgr::task`
- `sci_task_mgr::task_group`
- `sci_task_mgr::progress`

Important types:

- `ManagerConfig`
- `Task`
- `TaskContext`
- `TaskGroup`
- `ProgressHandle`
- `ProgressStore`

## Task Model

Concrete tasks implement `Task` and own their config directly.

The current contract is:

- `new(config)`
- `rebuild_from(config, checkpoint)`
- `config()`
- `evolve_one_epoch(&mut self, context)`

`TaskContext` currently provides:

- shared `TrajectoryHub` access
- progress reporting handle
- manager-assigned epoch number

## Task Groups

`TaskGroup<T>` is a generic epoch runner over `Vec<T>`.

It currently supports:

- constructing a group from tasks plus a shared `TrajectoryHub`
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
