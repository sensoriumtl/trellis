use tracing::{
    debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    Level, Span,
};

use crate::State;

use super::Watch;

pub struct TracingLogger {
    level: Level,
    full_span: Option<Span>,
    iter_span: Option<Span>,
}

impl TracingLogger {
    pub fn new(level: Level) -> Self {
        Self {
            level,
            iter_span: None,
            full_span: None,
        }
    }
}

impl<S: State> Watch<S> for TracingLogger {
    fn initialise_watcher(&mut self, name: &str) -> Result<(), super::WatchError> {
        let span = match self.level {
            Level::TRACE => trace_span!("outer span", name = name),
            Level::WARN => warn_span!("outer span", name = name),
            Level::INFO => info_span!("outer span", name = name),
            Level::ERROR => error_span!("outer span", name = name),
            Level::DEBUG => debug_span!("outer span", name = name),
        };
        span.enter();
        self.full_span = Some(span);
        Ok(())
    }

    fn watch_initialisation(
        &mut self,
        name: &str,
        kv: &crate::kv::KV,
    ) -> Result<(), super::WatchError> {
        for idx in &kv.kv {
            match self.level {
                Level::TRACE => trace!(key = idx.0, value = idx.1.to_string()),
                Level::WARN => warn!(key = idx.0, value = idx.1.to_string()),
                Level::INFO => info!(key = idx.0, value = idx.1.to_string()),
                Level::ERROR => error!(key = idx.0, value = idx.1.to_string()),
                Level::DEBUG => debug!(key = idx.0, value = idx.1.to_string()),
            }
        }
        Ok(())
    }

    fn watch_iteration(&mut self, state: &S, kv: &crate::kv::KV) -> Result<(), super::WatchError> {
        let iter = state.current_iteration();
        println!("{iter}");
        match self.level {
            Level::TRACE => trace!(iteration = state.current_iteration()),
            Level::WARN => warn!(iteration = state.current_iteration()),
            Level::INFO => info!(iteration = state.current_iteration()),
            Level::ERROR => error!(iteration = state.current_iteration()),
            Level::DEBUG => debug!(iteration = state.current_iteration()),
        }
        for idx in &kv.kv {
            match self.level {
                Level::TRACE => trace!(key = idx.0, value = idx.1.to_string()),
                Level::WARN => warn!(key = idx.0, value = idx.1.to_string()),
                Level::INFO => info!(key = idx.0, value = idx.1.to_string()),
                Level::ERROR => error!(key = idx.0, value = idx.1.to_string()),
                Level::DEBUG => debug!(key = idx.0, value = idx.1.to_string()),
            }
        }
        Ok(())
    }

    fn initialise_iteration(&mut self, name: &str) -> Result<(), super::WatchError> {
        let span = match self.level {
            Level::TRACE => trace_span!("iter span", name = name),
            Level::WARN => warn_span!("iter span", name = name),
            Level::INFO => info_span!("iter span", name = name),
            Level::ERROR => error_span!("iter span", name = name),
            Level::DEBUG => debug_span!("iter span", name = name),
        };
        span.enter();
        self.iter_span = Some(span);
        Ok(())
    }

    fn finalise_watcher(&mut self, _name: &str) -> Result<(), super::WatchError> {
        let _ = self.full_span.take();
        Ok(())
    }
}
