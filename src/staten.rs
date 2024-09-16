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
pub enum Status<E> {
    /// The solver has terminated for [`Reason`]
    Terminated(Reason<E>),
    /// The solver has not terminated
    NotTerminated,
}

impl<E> Default for Status<E> {
    fn default() -> Self {
        Self::NotTerminated
    }
}

/// Reasons a solver may terminate
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reason<E> {
    /// The caller has manually terminated the process with ctrl-C
    ControlC,
    /// A calling thread had terminated the process using a [`tokio::CancellationToken`]
    Controller,
    /// The solver has converged to the requested tolerance
    Converged,
    /// The solver has exceeded the maximum allowable iterations
    ExceededMaxIterations,
    /// An error was encountered during the solving procedure
    Error(E),
}

/// The state of the [`trellis`] solver
///
/// This contains generic fields common to all solvers, as well as a user-defined state
/// `S` which contains application specific fields.
pub struct State<S: UserState, E> {
    /// The specific component of the state implements the application specific code
    specific: S,
    /// The current iteration number
    iter: usize,
    /// The last iteration number where the smallest error estimate was found
    last_best_iter: usize,
    /// The maximum number of permitted iterations
    max_iter: usize,
    /// The time since the solver was instantiated
    time: Option<Duration>,
    /// The termination status of the solver
    termination_status: Status<E>,
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
pub trait CoreState<E> {
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
    fn terminate_due_to(self, reason: Reason<E>) -> Self;
    fn get_param(&self) -> Option<&Self::Param>;
    fn measure(&self) -> Self::Float;
    fn best_measure(&self) -> Self::Float;
    fn iterations_since_best(&self) -> usize;

    // Set functions
    fn relative_tolerance(self, tol: Self::Float) -> Self;
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

impl<S, E> CoreState<E> for State<S, E>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
    E: PartialEq,
{
    type Float = <S as UserState>::Float;
    type Param = <S as UserState>::Param;

    fn new() -> Self {
        Self {
            specific: S::new(),
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
        self.specific.is_initialised()
    }

    fn is_terminated(&self) -> bool {
        self.termination_status != Status::NotTerminated
    }

    fn terminate_due_to(mut self, reason: Reason<E>) -> Self {
        self.termination_status = Status::Terminated(reason);
        self
    }

    fn update(mut self) -> Self {
        let error_estimate = self.specific.update();
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

            self.specific.last_was_best();
        }

        if self.error < self.relative_tolerance {
            return self.terminate_due_to(Reason::Converged);
        }
        if self.current_iteration() > self.max_iter {
            return self.terminate_due_to(Reason::ExceededMaxIterations);
        }

        self
    }

    fn get_param(&self) -> Option<&Self::Param> {
        self.specific.get_param()
    }

    fn measure(&self) -> S::Float {
        self.error
    }

    fn best_measure(&self) -> S::Float {
        self.best_error
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
}
