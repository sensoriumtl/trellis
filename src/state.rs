use std::fmt::Display;

use hifitime::Duration;
use serde::Serialize;

pub trait TrellisFloat: Display + Serialize {}

impl TrellisFloat for f64 {}

#[derive(Eq, PartialEq)]
pub enum Status {
    Terminated(Reason),
    NotTerminated,
}

#[derive(Eq, PartialEq)]
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
