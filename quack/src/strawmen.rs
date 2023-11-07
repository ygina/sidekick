use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Strawman quACK implementation that echoes every packet identifier.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrawmanAQuack {
    pub sidecar_id: u32,
}

/// Strawman quACK implementation that echoes a sliding window of packet identifiers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrawmanBQuack {
    pub window: VecDeque<u32>,
    #[serde(skip)]
    pub window_size: usize,
}

impl StrawmanBQuack {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::new(),
            window_size,
        }
    }

    pub fn insert(&mut self, value: u32) {
        self.window.push_back(value);
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
    }
}
