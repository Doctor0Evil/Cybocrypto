use crate::frame_store::{FrameStore, StoredFrameRecord};
use crate::chat::AiChatFrame;
use crate::session_phoenix::DecisionLogEntry;
use crate::sovereign_chat::AiChatSovereignEnvelope;

#[derive(Debug)]
pub enum StreamError<E> {
    FrameStore(E),
    SeqMismatch { expected: u64, got: u64 },
}

pub struct DecisionToFrameStreamer<S: FrameStore> {
    pub session_id: String,
    pub next_expected_seq: u64,
    pub store: S,
}

impl<S: FrameStore> DecisionToFrameStreamer<S> {
    pub fn new(session_id: impl Into<String>, start_seq: u64, store: S) -> Self {
        Self {
            session_id: session_id.into(),
            next_expected_seq: start_seq,
            store,
        }
    }

    pub fn ingest(
        &mut self,
        decision: &DecisionLogEntry,
        envelope: &AiChatSovereignEnvelope,
    ) -> Result<(), StreamError<S::Error>> {
        if decision.seq != self.next_expected_seq {
            return Err(StreamError::SeqMismatch {
                expected: self.next_expected_seq,
                got: decision.seq,
            });
        }

        let record = StoredFrameRecord {
            session_id: self.session_id.clone(),
            seq: decision.seq,
            frame: envelope.frame.clone(),
        };

        self.store
            .append_frame(record)
            .map_err(StreamError::FrameStore)?;

        self.next_expected_seq += 1;
        Ok(())
    }
}
