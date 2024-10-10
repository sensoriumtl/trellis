use crate::Problem;

/// Trait implemented by all problems solveable by `Trellis`
///
/// A calculation defines the core loop of the solver. Typically we would write a for loop,
/// consisting of an initialisation step where the calculation is arranged, a procedure carried out
/// on each loop iteration, and a finalisation step prior to return. This trait separates these
/// methods so they can be called by the [`Runner`]
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

    /// An identifier for the calculation.
    ///
    /// This identifier is printed in tracing logs
    const NAME: &'static str;
    /// Initialisation.
    ///
    /// This step prepares the state object for the main calculation loop.
    fn initialise(&mut self, _problem: &mut Problem<P>, state: S) -> Result<S, Self::Error> {
        Ok(state)
    }
    /// One iteration of the core algorithm
    fn next(&mut self, problem: &mut Problem<P>, state: S) -> Result<S, Self::Error>;
    /// Converts the internal state to the user-facing return datatype
    fn finalise(&mut self, problem: &mut Problem<P>, state: S)
        -> Result<Self::Output, Self::Error>;
}
