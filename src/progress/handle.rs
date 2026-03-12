use std::sync::mpsc::Sender;

use super::{ProgressEvent, ProgressEventKind};

/// Cloneable task-scoped handle for emitting progress events.
#[derive(Clone)]
pub struct ProgressHandle {
    task_index: usize,
    epoch: u64,
    tx: Sender<ProgressEvent>,
}

impl ProgressHandle {
    /// Construct a progress handle for one task during one epoch.
    pub fn new(task_index: usize, epoch: u64, tx: Sender<ProgressEvent>) -> Self {
        Self {
            task_index,
            epoch,
            tx,
        }
    }

    /// Return the task index associated with this handle.
    pub fn task_index(&self) -> usize {
        self.task_index
    }

    /// Return the epoch associated with this handle.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Emit a raw progress event kind.
    pub fn report(&self, kind: ProgressEventKind) {
        let _ = self.tx.send(ProgressEvent {
            task_index: self.task_index,
            epoch: self.epoch,
            kind,
        });
    }

    /// Emit an `EpochStarted` event.
    pub fn epoch_started(&self) {
        self.report(ProgressEventKind::EpochStarted);
    }

    /// Emit an `EpochCompleted` event.
    pub fn epoch_completed(&self) {
        self.report(ProgressEventKind::EpochCompleted);
    }

    /// Emit a human-readable diagnostic message.
    pub fn message(&self, message: impl Into<String>) {
        self.report(ProgressEventKind::Message(message.into()));
    }
}
