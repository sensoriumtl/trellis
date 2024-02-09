mod builder;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use hifitime::{Duration, Epoch};

use crate::controller::{set_handler, Control};
use crate::{Calculation, Output, Problem, Reason, State};

pub type Error = Box<dyn std::error::Error>;

pub enum Caller {
    CtrlC,
    Controller,
}

pub struct Killswitch {
    caller: Caller,
    inner: Arc<AtomicBool>,
}

impl Killswitch {
    fn is_dead(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

/// General purpose calculation runner
pub struct Runner<C, P, S, R> {
    /// Calculation to be run
    calculation: C,
    /// The problem to solve
    problem: Problem<P>,
    /// Current state of the run
    state: Option<S>,
    /// Should execution be timed
    time: bool,
    /// Can we cancel with control c?
    control_c: bool,
    /// Receiver
    ///
    /// When a signal is received on this channel the calculation is terminated.
    controller: Option<R>,
    ///
    signals: Vec<Killswitch>,
}

impl<C, P, S, R> Runner<C, P, S, R> {
    fn now(&self) -> Result<Option<Epoch>, hifitime::errors::Errors> {
        if self.time {
            return Ok(Some(Epoch::now()?));
        }
        Ok(None)
    }

    fn duration_since(
        &self,
        maybe_epoch: Option<&Epoch>,
    ) -> Result<Option<Duration>, hifitime::errors::Errors> {
        if let Some(epoch) = maybe_epoch {
            let now = self.now()?.unwrap();
            return Ok(Some(now - *epoch));
        }
        Ok(None)
    }

    fn initialise_control_c(&mut self) -> Result<Arc<AtomicBool>, Error> {
        let received_kill_signal_from_control_c = Arc::new(AtomicBool::new(false));

        #[cfg(feature = "ctrlc")]
        {
            // Clone the state as the value needs to move into the closure
            let state = received_kill_signal_from_control_c.clone();
            ctrlc::set_handler(move || {
                state.store(true, Ordering::SeqCst);
            })?;
        }

        Ok(received_kill_signal_from_control_c)
    }
}

impl<C, P, S, R> Runner<C, P, S, R>
where
    C: Calculation<P, S>,
    S: State,
{
    fn kill_signal_received(&self) -> bool {
        self.signals
            .iter()
            .any(|signal| signal.inner.load(Ordering::SeqCst))
    }

    fn kill_cause(&self) -> Option<Reason> {
        self.signals
            .iter()
            .filter(|signal| signal.inner.load(Ordering::SeqCst))
            .next()
            .map(|signal| match signal.caller {
                Caller::CtrlC => Reason::ControlC,
                Caller::Controller => Reason::Controller,
            })
    }

    fn initialise(&mut self, mut state: S) -> Result<S, Error> {
        state = self.calculation.initialise(&mut self.problem, state)?;
        state.update();
        Ok(state)
    }

    fn once(&mut self, mut state: S, maybe_start_time: Option<&Epoch>) -> Result<S, Error> {
        let maybe_iteration_start_time = self.now()?;

        state = self.calculation.next(&mut self.problem, state)?;

        state.update();

        if let Some(total_duration) = self.duration_since(maybe_start_time)? {
            state.record_time(total_duration);
        }

        Ok(state)
    }

    fn finalise(&mut self, mut state: S) -> Result<S, Error> {
        state = self.calculation.finalise(&mut self.problem, state)?;
        state.update();
        Ok(state)
    }

    /// Execute the runner
    fn run(mut self) -> Result<Output<C, P, S>, Error> {
        // Todo: Load checkpoints?
        let start_time = self.now()?;

        let mut state = self.state.take().unwrap();

        // TODO: This only really matters is there is a checkpoint loaded, at the moment we have
        // none so the check is redundant
        state = if !state.is_initialised() {
            self.initialise(state)?
        } else {
            state
        };

        loop {
            if self.kill_signal_received() {
                state = state.terminate_due_to(self.kill_cause().unwrap());
                break;
            }
            state = self.once(state, start_time.as_ref())?;
        }

        state = self.finalise(state)?;

        Ok(Output::new(self.problem, self.calculation, state))
    }
}

impl<C, P, S, R> Runner<C, P, S, R>
where
    R: Control + 'static,
{
    fn initialise_kill_signal_handler(&mut self) -> Result<Arc<AtomicBool>, Error> {
        let received_kill_signal_from_controller = Arc::new(AtomicBool::new(false));

        // Clone the state as the value needs to move into the closure
        let state = received_kill_signal_from_controller.clone();
        set_handler(self.controller.take().unwrap(), move || {
            state.store(true, Ordering::SeqCst);
        })?;

        Ok(received_kill_signal_from_controller)
    }
}

pub trait InitialiseRunner {
    fn initialise_controllers(&mut self) -> Result<(), Error>;
}

impl<C, P, S> InitialiseRunner for Runner<C, P, S, ()> {
    fn initialise_controllers(&mut self) -> Result<(), Error> {
        let received_kill_signal_from_control_c = Killswitch {
            caller: Caller::CtrlC,
            inner: self.initialise_control_c()?,
        };
        self.signals = vec![received_kill_signal_from_control_c];
        Ok(())
    }
}

impl<C, P, S, R> InitialiseRunner for Runner<C, P, S, R>
where
    R: Control + 'static,
{
    fn initialise_controllers(&mut self) -> Result<(), Error> {
        let received_kill_signal_from_control_c = Killswitch {
            caller: Caller::CtrlC,
            inner: self.initialise_control_c()?,
        };

        let received_kill_signal_from_controller = Killswitch {
            caller: Caller::Controller,
            inner: self.initialise_kill_signal_handler()?,
        };
        self.signals = vec![
            received_kill_signal_from_control_c,
            received_kill_signal_from_controller,
        ];
        Ok(())
    }
}
