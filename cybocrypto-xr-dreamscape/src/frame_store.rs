use crate::chat::AiChatFrame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFrameRecord {
    pub session_id: String,
    pub seq: u64,
    pub frame: AiChatFrame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStoreQuery {
    pub session_id: String,
    pub from_seq: u64,
    pub to_seq: u64,
}

pub trait FrameStore {
    type Error;

    fn append_frame(&mut self, record: StoredFrameRecord) -> Result<(), Self::Error>;

    fn load_frames(&self, query: FrameStoreQuery) -> Result<Vec<StoredFrameRecord>, Self::Error>;
}
