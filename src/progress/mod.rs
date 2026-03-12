mod event;
mod handle;
mod store;

/// Immutable progress event emitted by tasks during execution.
pub use event::{ProgressEvent, ProgressEventKind};
/// Lightweight task-scoped emitter used to send progress events.
pub use handle::ProgressHandle;
/// In-memory store for collected progress events plus a constructor helper.
pub use store::{ProgressStore, new_progress_store};
