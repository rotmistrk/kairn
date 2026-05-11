//! MessageRing — ring buffer for application messages.

use std::collections::VecDeque;

pub use txv_core::message::{Message, MsgLevel};

const RING_CAPACITY: usize = 100;

/// Ring buffer holding the last N messages.
pub struct MessageRing {
    entries: VecDeque<Message>,
    /// Monotonic counter — incremented on each push.
    generation: u64,
}

impl Default for MessageRing {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRing {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(RING_CAPACITY),
            generation: 0,
        }
    }

    pub fn push(&mut self, msg: Message) {
        if let Some(last) = self.entries.back_mut() {
            if last.level == msg.level && last.origin == msg.origin && last.text == msg.text {
                last.count += 1;
                self.generation += 1;
                return;
            }
        }
        if self.entries.len() >= RING_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(msg);
        self.generation += 1;
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entries(&self) -> &VecDeque<Message> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_retrieve() {
        let mut ring = MessageRing::new();
        ring.push(Message::info("test", "hello"));
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.entries()[0].text, "hello");
        assert_eq!(ring.entries()[0].origin, "test");
    }

    #[test]
    fn ring_evicts_oldest() {
        let mut ring = MessageRing::new();
        for i in 0..110 {
            ring.push(Message::info("test", format!("msg {i}")));
        }
        assert_eq!(ring.len(), RING_CAPACITY);
        assert_eq!(ring.entries()[0].text, "msg 10");
    }

    #[test]
    fn generation_increments() {
        let mut ring = MessageRing::new();
        assert_eq!(ring.generation(), 0);
        ring.push(Message::info("x", "a"));
        assert_eq!(ring.generation(), 1);
        ring.push(Message::error("x", "b"));
        assert_eq!(ring.generation(), 2);
    }

    #[test]
    fn coalesces_repeated_messages() {
        let mut ring = MessageRing::new();
        ring.push(Message::info("git", "Staged: foo.rs"));
        ring.push(Message::info("git", "Staged: foo.rs"));
        ring.push(Message::info("git", "Staged: foo.rs"));
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.entries()[0].count, 3);
        assert_eq!(ring.generation(), 3);
    }

    #[test]
    fn different_messages_not_coalesced() {
        let mut ring = MessageRing::new();
        ring.push(Message::info("git", "Staged: foo.rs"));
        ring.push(Message::info("git", "Staged: bar.rs"));
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.entries()[0].count, 1);
        assert_eq!(ring.entries()[1].count, 1);
    }
}
