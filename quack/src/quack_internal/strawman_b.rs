use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrawmanBQuack {
    pub window: VecDeque<u32>,
}
