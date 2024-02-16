use crate::{kv::KV, Problem};

pub trait Calculation<P, S> {
    /// The error associated with the problem
    type Error: std::error::Error + 'static;

    const NAME: &'static str;
    /// Initialisation
    fn initialise(
        &mut self,
        problem: &mut Problem<P>,
        state: S,
    ) -> Result<(S, Option<KV>), Self::Error>;
    /// One iteration of the core algorithm
    fn next(&mut self, problem: &mut Problem<P>, state: S) -> Result<(S, Option<KV>), Self::Error>;
    /// Any steps to be taken on convergence
    fn finalise(
        &mut self,
        problem: &mut Problem<P>,
        state: S,
    ) -> Result<(S, Option<KV>), Self::Error>;
}
