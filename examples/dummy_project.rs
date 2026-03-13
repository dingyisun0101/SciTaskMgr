#[path = "dummy_project/dummy_task.rs"]
mod dummy_task;
#[path = "dummy_project/dummy_task_group.rs"]
mod dummy_task_group;

use std::path::PathBuf;

use dummy_task_group::TaskGroup;

fn main() -> std::io::Result<()> {
    // Keep generated example output close to the example source tree.
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("output")
        .join("dummy_project");
    let example_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("dummy_project");

    // Build and run one manager-owned task group from separate task-group and
    // task-config TOML files.
    let mut task_group = TaskGroup::new(&output_dir, &example_dir)?;
    let written = task_group.run()?;

    // Print a small summary so the example is easy to inspect manually.
    println!("dummy project evolved {} tasks", written.len());
    for path in written {
        println!("{}", path.display());
    }

    Ok(())
}
