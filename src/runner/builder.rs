use super::{Error, InitialiseRunner, Runner};
use crate::{
    watchers::{Frequency, Observable, Observer, ObserverVec},
    Calculation, Control, Problem, State,
};

pub trait GenerateBuilder<P, S>: Sized {
    fn build_for(self, problem: P) -> Builder<Self, P, S, ()>;
}

impl<C, P, S> GenerateBuilder<P, S> for C
where
    C: Calculation<P, S>,
    S: State,
{
    fn build_for(self, problem: P) -> Builder<Self, P, S, ()> {
        Builder {
            problem,
            calculation: self,
            state: S::new(),
            time: true,
            control_c: false,
            controller: (),
            observers: ObserverVec::default(),
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
    observers: ObserverVec<S>,
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

    #[must_use]
    pub fn attach_observer<OBS: Observer<S> + 'static>(
        mut self,
        observer: OBS,
        frequency: Frequency,
    ) -> Self {
        self.observers.attach(
            std::sync::Arc::new(std::sync::Mutex::new(observer)),
            frequency,
        );
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
            observers: self.observers,
        }
    }

    pub fn finalise(self) -> Result<Runner<C, P, S, ()>, Error> {
        let mut runner = Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            controller: None,
            signals: vec![],
            observers: self.observers,
        };
        runner.initialise_controllers()?;
        Ok(runner)
    }
}

impl<C, P, S, R> Builder<C, P, S, R>
where
    R: Control + 'static,
{
    pub fn finalise(self) -> Result<Runner<C, P, S, R>, Error> {
        let mut runner = Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            controller: Some(self.controller),
            signals: vec![],
            observers: self.observers,
        };
        runner.initialise_controllers()?;
        Ok(runner)
    }
}
