use crate::Problem;

pub trait Calculation<P, S> {
    type Error: std::error::Error + 'static;
    fn initialise(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
    fn next(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
    fn finalise(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
}
