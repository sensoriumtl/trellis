mod status;

use crate::TrellisFloat;

use num_traits::float::FloatCore;
use web_time::Duration;

pub use status::{Cause, Status};

/// The user-defined state must implement this trait to be used as part of the trellis calculation
/// loop
///
/// All other state methods are auto-implemented on a type wrapping the user-defined state.
pub trait UserState {
    type Float: TrellisFloat;
    type Param;

    /// Create a new instance of the user-defined state object
    fn new() -> Self;

    // Returns true when the state object is initialised correctly
    fn is_initialised(&self) -> bool {
        true
    }
    // Update the state object at the end of an iteration
    fn update(&mut self) -> ErrorEstimate<Self::Float>;
    // Returns the current parameter value, if one is assigned
    fn get_param(&self) -> Option<&Self::Param>;
    // Returns true if the last iteration was the best iteration seen so far
    fn last_was_best(&mut self);
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
    pub(crate) termination_status: Status,
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

#[repr(transparent)]
// Wrapping for error estimates during a calculation run
pub struct ErrorEstimate<F>(pub F);

impl<S> State<S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    /// Create a new instance of the iteration state
    pub(crate) fn new() -> Self {
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

    /// Record the time since the solver began
    pub(crate) fn record_time(&mut self, duration: Duration) {
        self.time = Some(duration);
    }

    pub(crate) fn duration(&self) -> Option<&Duration> {
        self.time.as_ref()
    }

    /// Increment the iteration count
    pub(crate) fn increment_iteration(&mut self) {
        self.iter += 1;
    }

    /// Returns the current iteration number
    pub(crate) fn current_iteration(&self) -> usize {
        self.iter
    }

    /// Returns the number of iterations since the best result was observed
    pub(crate) fn iterations_since_best(&self) -> usize {
        self.iter - self.last_best_iter
    }
    /// Returns true if the state has been initialised. This means a problem specific inner solver
    /// has been attached
    pub(crate) fn is_initialised(&self) -> bool {
        self.specific
            .as_ref()
            .map_or(false, |state| state.is_initialised())
    }

    /// Returns true if the termination status is [`Status::Terminated`]
    pub(crate) fn is_terminated(&self) -> bool {
        self.termination_status != Status::NotTerminated
    }

    /// Terminates the solver for [`Cause`]
    pub(crate) fn terminate_due_to(mut self, reason: Cause) -> Self {
        self.termination_status = Status::Terminated(reason);
        self
    }

    /// Returns Some if the solver is terminated, else returns None
    pub(crate) fn termination_cause(&self) -> Option<&Cause> {
        use Status::*;
        match &self.termination_status {
            NotTerminated => None,
            Terminated(cause) => Some(cause),
        }
    }

    #[must_use]
    /// Update the state, and the interan state
    pub(crate) fn update(mut self) -> Self {
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

    /// Returns the parameter vector from the inner state variable
    pub(crate) fn get_param(&self) -> Option<&S::Param> {
        self.specific
            .as_ref()
            .and_then(|specific| specific.get_param())
    }

    /// Returns the current measure of progress
    pub(crate) fn measure(&self) -> S::Float {
        self.error
    }

    /// Returns the best measure of progress
    pub(crate) fn best_measure(&self) -> S::Float {
        self.best_error
    }

    /// Removes the specific state from the state and returns it
    pub fn take_specific(&mut self) -> S {
        self.specific.take().unwrap()
    }

    #[must_use]
    /// Set the relative tolerance target
    pub fn relative_tolerance(mut self, relative_tolerance: S::Float) -> Self {
        self.relative_tolerance = relative_tolerance;
        self
    }

    #[must_use]
    /// Set the maximum allowable iteration count
    pub fn max_iters(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    #[must_use]
    /// Set the internal state object
    pub fn set_specific(mut self, specific: S) -> Self {
        self.specific = Some(specific);
        self
    }
}
