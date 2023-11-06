use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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

    pub fn insert(&mut self, number: u32) {
        self.window.push_back(number);
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
    }
}
