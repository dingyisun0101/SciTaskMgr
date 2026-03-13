use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use rayon::ThreadPool;
use sci_task_io::trajectory::TrajectoryHub;
use sci_task_io::trajectory::{Trajectory, TrajectoryWriteHandle};

use crate::progress::ProgressHandle;

/// Manager-owned runtime services exposed to a task during one epoch.
pub struct TaskContext<'a> {
    hub: &'a TrajectoryHub,
    progress: &'a ProgressHandle,
    epoch: u64,
    num_threads: usize,
    compute_pool: Arc<ThreadPool>,
}

impl<'a> TaskContext<'a> {
    /// Build a new task context for a single task execution step.
    pub fn new(
        hub: &'a TrajectoryHub,
        progress: &'a ProgressHandle,
        epoch: u64,
        num_threads: usize,
        compute_pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            hub,
            progress,
            epoch,
            num_threads,
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

    /// Submit one owned epoch trajectory to the shared hub.
    pub fn submit_trajectory(
        &self,
        trajectory: Trajectory,
        path: impl Into<PathBuf>,
    ) -> io::Result<()> {
        trajectory.send_to_hub(self.hub, path)
    }

    /// Submit one owned epoch trajectory and return a handle that can wait for completion.
    pub fn submit_trajectory_tracked(
        &self,
        trajectory: Trajectory,
        path: impl Into<PathBuf>,
    ) -> io::Result<TrajectoryWriteHandle> {
        trajectory.send_to_hub_tracked(self.hub, path)
    }
}
