/// Kind of progress event emitted by a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEventKind {
    /// Signals that the task has started work for the current epoch.
    EpochStarted,
    /// Signals that the task has completed work for the current epoch.
    EpochCompleted,
    /// Arbitrary human-readable diagnostic message from the task.
    Message(String),
}

/// One progress event tagged with task and epoch identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressEvent {
    /// Zero-based position of the task inside the current task group.
    pub task_index: usize,
    /// Epoch associated with the event.
    pub epoch: u64,
    /// Event payload describing what happened.
    pub kind: ProgressEventKind,
}
