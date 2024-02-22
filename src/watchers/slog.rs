use crate::watchers::Watch;
use crate::{kv::KV, state::State};
use slog::{debug, info, o, trace, Drain, Key, Level, Logger, Record, Serializer};
use slog_async::OverflowStrategy;

/// A logger using the [`slog`](https://crates.io/crates/slog) crate as backend.
#[derive(Clone)]
pub struct SlogLogger {
    /// the logger
    logger: Logger,
    level: Level,
}

impl SlogLogger {
    /// Log to the logger given
    ///
    /// We often want to have behaviour delegated by the caller, for example in a complex
    /// application the log level and drain may be globally determined.
    ///
    /// This uses the parent logger implementation, rather than creating a new one.
    pub fn using(logger: &Logger, level: Level) -> Self {
        if matches!(level, Level::Error | Level::Warning) {
            panic!("we won't emit non-error messages at ERROR or WARNING...");
        }
        Self {
            logger: logger.clone(),
            level,
        }
    }

    /// Log to the terminal.
    ///
    /// Will block execution when buffer is full.
    pub fn terminal(level: Level) -> Self {
        if matches!(level, Level::Error | Level::Warning) {
            panic!("we won't emit non-error messages at ERROR or WARNING...");
        }
        let logger = Self::term_internal(OverflowStrategy::Block);
        Self { logger, level }
    }

    /// Log to the terminal without blocking execution.
    ///
    /// Messages may be lost in case of buffer overflow.
    ///
    pub fn terminal_noblock(level: Level) -> Self {
        if matches!(level, Level::Error | Level::Warning) {
            panic!("we won't emit non-error messages at ERROR or WARNING...");
        }
        let logger = Self::term_internal(OverflowStrategy::Drop);
        Self { logger, level }
    }

    /// Create terminal logger with a given `OverflowStrategy`.
    fn term_internal(overflow_strategy: OverflowStrategy) -> Logger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator)
            .use_original_order()
            .build()
            .fuse();
        let drain = slog_async::Async::new(drain)
            .overflow_strategy(overflow_strategy)
            .build()
            .fuse();
        slog::Logger::root(drain, o!())
    }
}

struct LogState<I>(I);

impl<I> slog::KV for LogState<&'_ I>
where
    I: State,
{
    fn serialize(&self, _record: &Record, serializer: &mut dyn Serializer) -> slog::Result {
        serializer.emit_str(Key::from("measure"), &self.0.measure().to_string())?;
        serializer.emit_str(
            Key::from("best measure"),
            &self.0.best_measure().to_string(),
        )?;
        serializer.emit_usize(Key::from("iter"), self.0.current_iteration())?;
        serializer.emit_usize(Key::from("iter since best"), self.0.iterations_since_best())?;
        Ok(())
    }
}

impl slog::KV for KV {
    fn serialize(&self, _record: &Record, serializer: &mut dyn Serializer) -> slog::Result {
        for idx in &self.kv {
            serializer.emit_str(Key::from(*idx.0), &idx.1.to_string())?;
        }
        Ok(())
    }
}

impl<S> Watch<S> for SlogLogger
where
    S: State,
{
    /// Log basic information about the optimization after initialization.
    fn watch_initialisation(&mut self, name: &str, kv: &KV) -> Result<(), super::WatchError> {
        match self.level {
            Level::Info => info!(self.logger, "starting: {}", name; kv),
            Level::Debug => debug!(self.logger, "starting: {}", name; kv),
            Level::Trace => trace!(self.logger, "starting: {}", name; kv),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn watch_finalisation(&mut self, name: &str, kv: &KV) -> Result<(), super::WatchError> {
        match self.level {
            Level::Info => info!(self.logger, "finished: {}", name; kv),
            Level::Debug => debug!(self.logger, "finished: {}", name; kv),
            Level::Trace => trace!(self.logger, "finished: {}", name; kv),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn watch_iteration(&mut self, state: &S, kv: &KV) -> Result<(), super::WatchError> {
        match self.level {
            Level::Info => info!(self.logger, ""; LogState(state), kv),
            Level::Debug => debug!(self.logger, ""; LogState(state), kv),
            Level::Trace => trace!(self.logger, ""; LogState(state), kv),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }
}
