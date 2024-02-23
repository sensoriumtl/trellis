use std::fmt::Display;

use hifitime::Duration;
use serde::{Deserialize, Serialize};

pub trait TrellisFloat: Display + Serialize {}

impl TrellisFloat for f64 {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Status {
    Terminated(Reason),
    NotTerminated,
}

impl Default for Status {
    fn default() -> Self {
        Self::NotTerminated
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    ControlC,
    Controller,
    ExceededMaxIterations,
}

pub trait State {
    type Float: TrellisFloat;
    type Param;
    fn new() -> Self;
    fn record_time(&mut self, duration: Duration);
    fn increment_iteration(&mut self);
    fn current_iteration(&self) -> usize;
    fn update(&mut self);
    fn is_initialised(&self) -> bool;
    fn is_terminated(&self) -> bool;
    fn terminate_due_to(self, reason: Reason) -> Self;
    fn get_param(&self) -> Option<&Self::Param>;
    fn measure(&self) -> Self::Float;
    fn best_measure(&self) -> Self::Float;
    fn iterations_since_best(&self) -> usize;
}
