// use std::ops::{Sub, SubAssign, MulAssign, AddAssign};
// use std::fmt::{Debug, Display};
// use crate::arithmetic::{
//     ModularArithmetic,
//     ModularInteger,
//     MonicPolynomialEvaluator,
// };
// use crate::Quack;
use serde::{Deserialize, Serialize};
// use log::{debug, info, trace};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrawmanAQuack {
    pub sidecar_id: u32,
}
