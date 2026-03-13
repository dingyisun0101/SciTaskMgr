use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rayon::ThreadPool;
use sci_task_io::trajectory::TrajectoryHub;
use sci_task_io::trajectory::{Trajectory, TrajectoryWriteHandle};

use crate::progress::ProgressHandle;

/// Manager-owned runtime services exposed to a task during one epoch.
pub struct TaskContext<'a> {
    hub: &'a TrajectoryHub,
    progress: &'a ProgressHandle,
    task_index: usize,
    epoch: u64,
    num_threads: usize,
    trajectory_dir: PathBuf,
    submission_counter: Arc<AtomicU64>,
    compute_pool: Arc<ThreadPool>,
}

impl<'a> TaskContext<'a> {
    /// Build a new task context for a single task execution step.
    pub fn new(
        hub: &'a TrajectoryHub,
        progress: &'a ProgressHandle,
        task_index: usize,
        epoch: u64,
        num_threads: usize,
        trajectory_dir: PathBuf,
        submission_counter: Arc<AtomicU64>,
        compute_pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            hub,
            progress,
            task_index,
            epoch,
            num_threads,
            trajectory_dir,
            submission_counter,
            compute_pool,
        }
    }

    /// Return the shared trajectory hub owned by the task group.
    pub fn hub(&self) -> &TrajectoryHub {
        self.hub
    }

    /// Return the progress emitter scoped to this task and epoch.
    pub fn progress(&self) -> &ProgressHandle {
        self.progress
    }

    /// Return the zero-based task index assigned by the manager.
    pub fn task_index(&self) -> usize {
        self.task_index
    }

    /// Return the epoch number assigned by the task group.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Return the maximum number of threads this task may use for internal parallel work.
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }

    /// Run work inside the task-scoped Rayon pool configured by the manager.
    pub fn install_compute_pool<OP, R>(&self, op: OP) -> R
    where
        OP: FnOnce() -> R + Send,
        R: Send,
    {
        self.compute_pool.install(op)
    }

    /// Return the task-scoped trajectory directory assigned by the manager.
    pub fn trajectory_dir(&self) -> &std::path::Path {
        &self.trajectory_dir
    }

    /// Submit one owned epoch trajectory to the shared hub under a manager-derived path.
    pub fn submit_trajectory(&self, label: impl AsRef<str>, trajectory: Trajectory) -> io::Result<()> {
        trajectory.send_to_hub(self.hub, self.next_trajectory_path(label.as_ref()))
    }

    /// Submit one owned epoch trajectory and return a handle that can wait for completion.
    pub fn submit_trajectory_tracked(
        &self,
        label: impl AsRef<str>,
        trajectory: Trajectory,
    ) -> io::Result<TrajectoryWriteHandle> {
        trajectory.send_to_hub_tracked(self.hub, self.next_trajectory_path(label.as_ref()))
    }

    /// Build the next manager-owned trajectory path for this task and epoch.
    fn next_trajectory_path(&self, label: &str) -> PathBuf {
        let submission_index = self.submission_counter.fetch_add(1, Ordering::Relaxed);
        self.trajectory_dir.join(format!(
            "epoch_{:010}/{}_{}.json",
            self.epoch,
            sanitize_label(label),
            submission_index
        ))
    }
}

/// Sanitize a user-provided trajectory label for use as a filename stem.
fn sanitize_label(label: &str) -> String {
    let sanitized: String = label
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect();
    if sanitized.is_empty() {
        "trajectory".to_string()
    } else {
        sanitized
    }
}
