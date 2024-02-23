use tracing::{debug, info, trace, Level, Value};

use crate::watchers::{ObservationError, Observer, Stage};
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

impl<F: tracing::Value, S: State<Float = F>> Observer<S> for Tracer {
    fn observe(&self, ident: &'static str, subject: &S, key_value: Option<&KV>, stage: Stage) {
        match stage {
            Stage::Initialisation => self.observe_initialisation(ident, key_value),
            Stage::Finalisation => self.observe_finalisation(ident, key_value),
            Stage::Iteration => self.observe_iteration(subject, key_value),
        }
        .unwrap()
    }
}

impl Tracer {
    /// Log basic information about the optimization after initialization.
    fn observe_initialisation(&self, name: &str, _kv: Option<&KV>) -> Result<(), ObservationError> {
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

    fn observe_finalisation(&self, name: &str, _kv: Option<&KV>) -> Result<(), ObservationError> {
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

    fn observe_iteration<F, S>(&self, state: &S, _kv: Option<&KV>) -> Result<(), ObservationError>
    where
        S: State<Float = F>,
        F: Value,
    {
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
