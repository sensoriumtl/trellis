use std::fmt::Display;

use hifitime::Duration;
use num_traits::float::FloatCore;
use serde::{Deserialize, Serialize};

/// Core trait a float must satisfy for the trellis calculation loop to progress
pub trait TrellisFloat: Display + Serialize {}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}

/// The status of the solver
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Status {
    /// The solver has terminated for [`Cause`]
    Terminated(Cause),
    /// The solver has not terminated
    NotTerminated,
}

impl Default for Status {
    fn default() -> Self {
        Self::NotTerminated
    }
}

/// Causes a solver may terminate
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cause {
    /// The caller has manually terminated the process with ctrl-C
    ControlC,
    /// A calling thread had terminated the process using a [`tokio::CancellationToken`]
    Controller,
    /// The solver has converged to the requested tolerance
    Converged,
    /// The solver has exceeded the maximum allowable iterations
    ExceededMaxIterations,
}

/// The state of the [`trellis`] solver
///
/// This contains generic fields common to all solvers, as well as a user-defined state
/// `S` which contains application specific fields.
pub struct State<S: UserState> {
    /// The specific component of the state implements the application specific code
    specific: Option<S>,
    /// The current iteration number
    iter: usize,
    /// The last iteration number where the smallest error estimate was found
    last_best_iter: usize,
    /// The maximum number of permitted iterations
    max_iter: usize,
    /// The time since the solver was instantiated
    time: Option<Duration>,
    /// The termination status of the solver
    termination_status: Status,
    /// The current estimate of the error, that observed in the previous iteration
    error: S::Float,
    /// The estimate of the error observed in the one before last iteration
    prev_error: S::Float,
    /// The best value of the error observed during the entire calculation
    best_error: S::Float,
    /// The second best value of the error observed during the entire calculation
    prev_best_error: S::Float,
    /// The target relative tolerance
    relative_tolerance: S::Float,
}

/// Generic methods the state of a solver must satisfy for the core function loop to progress.
pub trait CoreState {
    type Specific: UserState;
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
    fn terminate_due_to(self, reason: Cause) -> Self;
    fn termination_cause(&self) -> Option<&Cause>;
    fn get_param(&self) -> Option<&<Self::Specific as UserState>::Param>;
    fn measure(&self) -> <Self::Specific as UserState>::Float;
    fn best_measure(&self) -> <Self::Specific as UserState>::Float;
    fn iterations_since_best(&self) -> usize;
    fn take_specific(&mut self) -> Self::Specific;
    fn set_specific(self, specific: Self::Specific) -> Self;

    // Set functions
    fn relative_tolerance(self, tol: <Self::Specific as UserState>::Float) -> Self;
    fn max_iters(self, max_iter: usize) -> Self;
}

pub struct ErrorEstimate<F>(pub F);

pub trait UserState {
    type Float: TrellisFloat;
    type Param;
    fn new() -> Self;
    fn is_initialised(&self) -> bool;
    fn update(&mut self) -> ErrorEstimate<Self::Float>;
    fn get_param(&self) -> Option<&Self::Param>;
    fn last_was_best(&mut self);
}

impl<S> CoreState for State<S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    type Specific = S;

    fn new() -> Self {
        Self {
            specific: Some(S::new()),
            iter: 0,
            last_best_iter: 0,
            max_iter: usize::MAX,
            termination_status: Status::NotTerminated,
            time: None,
            relative_tolerance: <<S as UserState>::Float as FloatCore>::epsilon(),
            error: <<S as UserState>::Float as FloatCore>::infinity(),
            prev_error: <<S as UserState>::Float as FloatCore>::infinity(),
            best_error: <<S as UserState>::Float as FloatCore>::infinity(),
            prev_best_error: <<S as UserState>::Float as FloatCore>::infinity(),
        }
    }

    fn record_time(&mut self, duration: Duration) {
        self.time = Some(duration);
    }

    fn increment_iteration(&mut self) {
        self.iter += 1;
    }

    fn current_iteration(&self) -> usize {
        self.iter
    }

    fn iterations_since_best(&self) -> usize {
        self.iter - self.last_best_iter
    }

    fn is_initialised(&self) -> bool {
        self.specific.is_some()
    }

    fn is_terminated(&self) -> bool {
        self.termination_status != Status::NotTerminated
    }

    fn terminate_due_to(mut self, reason: Cause) -> Self {
        self.termination_status = Status::Terminated(reason);
        self
    }

    fn termination_cause(&self) -> Option<&Cause> {
        use Status::*;
        match &self.termination_status {
            NotTerminated => None,
            Terminated(cause) => Some(cause),
        }
    }

    #[must_use]
    fn update(mut self) -> Self {
        let mut specific = self.specific.take().unwrap();
        let error_estimate = specific.update();
        self.error = error_estimate.0;
        if self.error < self.best_error
            || (FloatCore::is_infinite(self.error)
                && FloatCore::is_infinite(self.best_error)
                && FloatCore::is_sign_positive(self.error)
                    == FloatCore::is_sign_positive(self.best_error))
        {
            std::mem::swap(&mut self.prev_best_error, &mut self.best_error);
            self.best_error = self.error;
            self.last_best_iter = self.iter;

            specific.last_was_best();
        }
        self.specific = Some(specific);

        if self.error < self.relative_tolerance {
            return self.terminate_due_to(Cause::Converged);
        }
        if self.current_iteration() > self.max_iter {
            return self.terminate_due_to(Cause::ExceededMaxIterations);
        }

        self
    }

    fn get_param(&self) -> Option<&S::Param> {
        self.specific
            .as_ref()
            .map(|specific| specific.get_param())
            .flatten()
    }

    fn measure(&self) -> S::Float {
        self.error
    }

    fn best_measure(&self) -> S::Float {
        self.best_error
    }

    fn take_specific(&mut self) -> S {
        self.specific.take().unwrap()
    }

    #[must_use]
    fn relative_tolerance(mut self, relative_tolerance: S::Float) -> Self {
        self.relative_tolerance = relative_tolerance;
        self
    }

    #[must_use]
    fn max_iters(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    #[must_use]
    fn set_specific(mut self, specific: S) -> Self {
        self.specific = Some(specific);
        self
    }
}
