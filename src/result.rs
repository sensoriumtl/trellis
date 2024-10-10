//! This module defines the default output type for a trellis calculation, in addition to the error
//! wrapper.

use crate::{State, UserState};

/// The output of a calculation
///
/// The calculation output is user defined in the finalise step of the [`Calculation`] trait, but
/// this is presented as a good verbose option in situations where the caller wants granular
/// information about the calculation and its progress. It returns the entire original problem,
/// solver and state object.
pub struct Output<R, S>
where
    S: UserState,
{
    /// The original calculation carried out by `trellis`
    pub result: R,
    /// Solver state after the last iterationn
    pub state: State<S>,
}

impl<R, S> Output<R, S>
where
    S: UserState,
{
    pub(crate) fn new(result: R, state: State<S>) -> Self {
        Self { result, state }
    }
}

#[derive(thiserror::Error, Debug)]
/// An error wrapper for trellis calculations
///
/// The error wraps the underlying error type [`ErrorCause`], which contains information about the
/// reason the calculation failed. In addition it can optionally return the an output from the
/// calculation. This is useful in situations where a failure occured due to running out of
/// iterations, or termination from the parent thread, but the state of the calculation at that
/// point may still contain meaningful information. Maybe the calculation ran out of iterations
/// because it was unable to reach the required tolerance, but is still at convergence?
pub struct TrellisError<O, E> {
    #[source]
    /// The underlying error cause.
    pub cause: ErrorCause<E>,
    /// An optional result which can be extracted by the caller
    pub result: Option<O>,
}

impl<O, E> From<E> for TrellisError<O, E> {
    fn from(cause: E) -> Self {
        Self {
            cause: ErrorCause::User(cause),
            result: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorCause<E> {
    #[error("error in user defined calculation: {0}")]
    User(#[from] E),
    #[error("exceeded maximum number of iterations")]
    MaxIterExceeded,
    #[error("calculation cancelled due to ctrl-c")]
    ControlC,
    #[error("calculation cancelled due to cancelled token")]
    CancellationToken,
}
