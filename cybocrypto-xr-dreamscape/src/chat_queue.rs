use crate::chat::AiChatFrame;
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct AiChatQueue {
    frames: VecDeque<AiChatFrame>,
    last_seq: u64,
}

impl AiChatQueue {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            last_seq: 0,
        }
    }

    pub fn push(&mut self, frame: AiChatFrame) -> u64 {
        self.last_seq += 1;
        self.frames.push_back(frame);
        self.last_seq
    }

    pub fn pop_all(&mut self) -> Vec<AiChatFrame> {
        let mut out = Vec::with_capacity(self.frames.len());
        while let Some(frame) = self.frames.pop_front() {
            out.push(frame);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }
}
