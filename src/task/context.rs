use sci_task_io::trajectory::TrajectoryHub;

use crate::progress::ProgressHandle;

/// Manager-owned runtime services exposed to a task during one epoch.
pub struct TaskContext<'a> {
    hub: &'a TrajectoryHub,
    progress: &'a ProgressHandle,
    epoch: u64,
}

impl<'a> TaskContext<'a> {
    /// Build a new task context for a single task execution step.
    pub fn new(hub: &'a TrajectoryHub, progress: &'a ProgressHandle, epoch: u64) -> Self {
        Self {
            hub,
            progress,
            epoch,
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
}
