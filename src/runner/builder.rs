use super::Runner;
use crate::{Calculation, Control, Problem, State};

pub trait GenerateBuilder<P, S>: Sized {
    fn builder(self, problem: P) -> Builder<Self, P, S, ()>;
}

impl<C, P, S> GenerateBuilder<P, S> for C
where
    C: Calculation<P, S>,
    S: State,
{
    fn builder(self, problem: P) -> Builder<Self, P, S, ()> {
        Builder {
            problem,
            calculation: self,
            state: S::new(),
            time: true,
            control_c: false,
            controller: (),
        }
    }
}

pub struct Builder<C, P, S, R> {
    calculation: C,
    problem: P,
    state: S,
    time: bool,
    control_c: bool,
    controller: R,
}
impl<C, P, S, R> Builder<C, P, S, R> {
    #[must_use]
    pub fn control_c(mut self, control_c: bool) -> Self {
        self.control_c = control_c;
        self
    }

    #[must_use]
    pub fn time(mut self, time: bool) -> Self {
        self.time = time;
        self
    }

    /// Configure the attached state.
    ///
    /// Apply any runtime configuration option to the attached state.
    #[must_use]
    pub fn configure<F: FnOnce(S) -> S>(mut self, configure: F) -> Self {
        let state = configure(self.state);
        self.state = state;
        self
    }
}

impl<C, P, S> Builder<C, P, S, ()> {
    #[must_use]
    pub fn with_controller<R>(self, controller: R) -> Builder<C, P, S, R> {
        Builder {
            calculation: self.calculation,
            problem: self.problem,
            state: self.state,
            time: self.time,
            control_c: self.control_c,
            controller,
        }
    }

    pub fn build(self) -> Runner<C, P, S, ()> {
        Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            controller: None,
            signals: vec![],
        }
    }
}

impl<C, P, S, R> Builder<C, P, S, R>
where
    R: Control,
{
    pub fn build(self) -> Runner<C, P, S, R> {
        Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            controller: Some(self.controller),
            signals: vec![],
        }
    }
}
