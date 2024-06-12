use crate::Problem;

/// Trait implemented by all problems solved by `Trellis`
pub trait Calculation<P, S> {
    /// The error associated with the problem
    type Error: std::error::Error + 'static;
    /// The type returned to the caller.
    ///
    /// Trellis defines a data-rich [`Output`], which can be constructed from the calculation, and
    /// internal state. In some circumstances it may be appropriate to return this type to the
    /// caller. In other circumstances it may be preferential to bury this complexity, returning
    /// the caller a custom datatype.
    type Output;

    const NAME: &'static str;
    /// Initialisation.
    ///
    /// This step prepares the state object for the main calculation loop.
    fn initialise(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
    /// One iteration of the core algorithm
    fn next(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
    /// Converts the internal state to the return datatype
    fn finalise(&mut self, problem: &mut Problem<P>, state: S)
        -> Result<Self::Output, Self::Error>;
}
