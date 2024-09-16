use crate::Problem;

#[derive(Debug)]
pub struct Output<C, P, S> {
    /// calculation
    pub calculation: C,
    /// Problem to apply calculation to
    pub problem: Problem<P>,
    /// Iteration state
    pub state: S,
}

impl<C, P, S> Output<C, P, S> {
    pub(crate) fn new(problem: Problem<P>, calculation: C, state: S) -> Self {
        Self {
            problem,
            calculation,
            state,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct TrellisError<O, E> {
    #[source]
    pub cause: ErrorCause<E>,
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
