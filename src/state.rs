use std::fmt::Display;

use hifitime::Duration;
use serde::{Deserialize, Serialize};

pub trait TrellisFloat: Display + Serialize {}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}

/// The status of the solver
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Status {
    /// The solver has terminated for [`Reason`]
    Terminated(Reason),
    /// The solver has not terminated
    NotTerminated,
}

impl Default for Status {
    fn default() -> Self {
        Self::NotTerminated
    }
}

/// Reasons a solver may terminate
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    /// The caller has manually terminated the process with ctrl-C
    ControlC,
    /// A calling thread had terminated the process using a [`tokio::CancellationToken`]
    Controller,
    /// The solver has converged to the requested tolerance
    Converged,
    /// The solver has exceeded the maximum allowable iterations
    ExceededMaxIterations,
}

/// Generic methods the state of a solver must satisfy for the core function loop to progress.
pub trait State {
    type Float: TrellisFloat;
    type Param;
    /// Create a new instance of the iteration state
    fn new() -> Self;
    /// Record the time since the solver began
    fn record_time(&mut self, duration: Duration);
    /// Increment the iteration count
    fn increment_iteration(&mut self);
    fn current_iteration(&self) -> usize;
    fn update(self) -> Self;
    fn is_initialised(&self) -> bool;
    fn is_terminated(&self) -> bool;
    fn terminate_due_to(self, reason: Reason) -> Self;
    fn get_param(&self) -> Option<&Self::Param>;
    fn measure(&self) -> Self::Float;
    fn best_measure(&self) -> Self::Float;
    fn iterations_since_best(&self) -> usize;
}
