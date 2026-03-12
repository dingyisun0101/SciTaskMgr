use std::sync::mpsc::{self, Receiver, Sender};

use super::ProgressEvent;

/// In-memory collector for progress events emitted by tasks.
pub struct ProgressStore {
    rx: Receiver<ProgressEvent>,
    events: Vec<ProgressEvent>,
}

impl ProgressStore {
    /// Create a new store together with its paired sending endpoint.
    pub fn new() -> (Sender<ProgressEvent>, Self) {
        let (tx, rx) = mpsc::channel();
        (
            tx,
            Self {
                rx,
                events: Vec::new(),
            },
        )
    }

    /// Drain all currently queued events into the in-memory snapshot.
    pub fn drain(&mut self) {
        for event in self.rx.try_iter() {
            self.events.push(event);
        }
    }

    /// Return the collected event snapshot accumulated so far.
    pub fn snapshot(&self) -> &[ProgressEvent] {
        &self.events
    }
}

/// Convenience constructor for a progress sender/store pair.
pub fn new_progress_store() -> (Sender<ProgressEvent>, ProgressStore) {
    ProgressStore::new()
}
