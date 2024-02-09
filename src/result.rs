use crate::Problem;

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
