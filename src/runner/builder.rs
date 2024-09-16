use num_traits::float::FloatCore;

use super::{Error, InitialiseRunner, Runner};
use crate::{
    watchers::{Frequency, Observable, Observer, ObserverVec},
    Calculation, Control, CoreState, Problem, State, UserState,
};

pub trait GenerateBuilder<P, S>: Sized + Calculation<P, S>
where
    S: UserState,
{
    fn build_for(self, problem: P) -> Builder<Self, P, S, ()>;
}

impl<C, P, S> GenerateBuilder<P, S> for C
where
    C: Calculation<P, S>,
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    fn build_for(self, problem: P) -> Builder<Self, P, S, ()> {
        Builder {
            problem,
            calculation: self,
            state: State::new(),
            time: true,
            control_c: false,
            cancellation_token: (),
            observers: ObserverVec::default(),
        }
    }
}

pub struct Builder<C, P, S, T>
where
    C: Calculation<P, S>,
    S: UserState,
{
    calculation: C,
    problem: P,
    state: State<S>,
    time: bool,
    control_c: bool,
    cancellation_token: T,
    observers: ObserverVec<State<S>>,
}
impl<C, P, S, R> Builder<C, P, S, R>
where
    C: Calculation<P, S>,
    S: UserState,
{
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
    pub fn configure<F: FnOnce(State<S>) -> State<S>>(mut self, configure: F) -> Self {
        let state = configure(self.state);
        self.state = state;
        self
    }

    #[must_use]
    pub fn attach_observer<OBS: Observer<State<S>> + 'static>(
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

impl<C, P, S> Builder<C, P, S, ()>
where
    C: Calculation<P, S>,
    S: UserState,
{
    #[must_use]
    pub fn with_cancellation_token<T>(self, cancellation_token: T) -> Builder<C, P, S, T> {
        Builder {
            calculation: self.calculation,
            problem: self.problem,
            state: self.state,
            time: self.time,
            control_c: self.control_c,
            cancellation_token,
            observers: self.observers,
        }
    }

    pub fn finalise(self) -> Result<Runner<C, P, S, ()>, Error>
    where
        C: Calculation<P, S>,
        S: UserState,
    {
        let mut runner = Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            cancellation_token: None,
            signals: vec![],
            observers: self.observers,
        };
        runner.initialise_controllers()?;
        Ok(runner)
    }
}

impl<C, P, S, T> Builder<C, P, S, T>
where
    T: Control + 'static,
    C: Calculation<P, S>,
    S: UserState,
{
    pub fn finalise(self) -> Result<Runner<C, P, S, T>, Error> {
        let mut runner = Runner {
            problem: Problem::new(self.problem),
            calculation: self.calculation,
            state: Some(self.state),
            time: self.time,
            control_c: self.control_c,
            cancellation_token: Some(self.cancellation_token),
            signals: vec![],
            observers: self.observers,
        };
        runner.initialise_controllers()?;
        Ok(runner)
    }
}
