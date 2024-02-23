use tracing::{debug, info, trace, Level, Value};

use crate::watchers::{ObservationError, Observer, Stage, Subject};
use crate::{kv::KV, state::State};

/// A logger using the [`slog`](https://crates.io/crates/slog) crate as backend.
#[derive(Clone)]
pub struct Tracer {
    /// the logger
    level: Level,
}

impl Tracer {
    pub fn new(level: Level) -> Self {
        if matches!(level, Level::ERROR | Level::WARN) {
            panic!("we won't emit non-error messages at ERROR or WARN...");
        }
        Self { level }
    }
}

struct TracingState<I>(I);

impl<'a, S: State> Observer for Tracer {
    type Subject = Subject<'a, S>;
    fn observe(&self, subject: &Self::Subject) {
        match subject.stage {
            Stage::Initialisation => self.watch_initialisation(subject.name, subject.key_value),
            Stage::Finalisation => self.watch_finalisation(subject.name, subject.key_value),
            Stage::Iteration => self.watch_iteration(subject.state, subject.key_value),
        }
    }
}

impl<F, S> Tracer
where
    S: State<Float = F>,
    F: Value,
{
    /// Log basic information about the optimization after initialization.
    fn watch_initialisation(&mut self, name: &str, _kv: &KV) -> Result<(), ObservationError> {
        match self.level {
            Level::INFO => info!("initialising: {}", name),
            Level::DEBUG => debug!("initialising: {}", name),
            Level::TRACE => trace!("initialising: {}", name),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn watch_finalisation(&mut self, name: &str, _kv: &KV) -> Result<(), ObservationError> {
        match self.level {
            Level::INFO => info!("initialising: {}", name),
            Level::DEBUG => debug!("initialising: {}", name),
            Level::TRACE => trace!("initialising: {}", name),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn watch_iteration(&mut self, state: &S, _kv: &KV) -> Result<(), ObservationError> {
        match self.level {
            Level::INFO => info!(
                iteration = state.current_iteration(),
                best_measure = state.best_measure(),
                measure = state.measure(),
                since_best = state.iterations_since_best(),
            ),
            Level::DEBUG => debug!(
                iteration = state.current_iteration(),
                best_measure = state.best_measure(),
                measure = state.measure(),
                since_best = state.iterations_since_best(),
            ),
            Level::TRACE => trace!(
                iteration = state.current_iteration(),
                best_measure = state.best_measure(),
                measure = state.measure(),
                since_best = state.iterations_since_best(),
            ),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }
}
