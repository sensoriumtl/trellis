use std::sync::{Arc, Mutex};

use crate::{kv::KV, State};

#[cfg(feature = "slog")]
mod slog;
#[cfg(feature = "tracing")]
mod tracing;

#[cfg(feature = "tracing")]
pub use tracing::TracingLogger;

pub(crate) struct Watchers<S> {
    watchers: Vec<(Arc<Mutex<dyn Watch<S>>>, Frequency)>,
}

impl<S> Default for Watchers<S> {
    fn default() -> Self {
        Self { watchers: vec![] }
    }
}

impl<S> Watchers<S> {
    pub(crate) fn add<W: Watch<S> + 'static>(mut self, watcher: W, frequency: Frequency) -> Self {
        self.watchers
            .push((Arc::new(Mutex::new(watcher)), frequency));
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum WatchError {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Frequency {
    Never,
    Always,
    Every(usize),
    Last,
}

impl Default for Frequency {
    fn default() -> Self {
        Self::Never
    }
}

pub(crate) trait Watch<S> {
    fn initialise_watcher(&mut self, _name: &str) -> Result<(), WatchError> {
        Ok(())
    }

    fn watch_initialisation(&mut self, _name: &str, _kv: &KV) -> Result<(), WatchError> {
        Ok(())
    }

    fn watch_finalisation(&mut self, _state: &S, _kv: &KV) -> Result<(), WatchError> {
        Ok(())
    }

    fn watch_iteration(&mut self, _state: &S, _kv: &KV) -> Result<(), WatchError> {
        Ok(())
    }

    fn initialise_iteration(&mut self, _name: &str) -> Result<(), WatchError> {
        Ok(())
    }

    fn finalise_watcher(&mut self, _name: &str) -> Result<(), WatchError> {
        Ok(())
    }
}

impl<S: State> Watch<S> for Watchers<S> {
    fn watch_initialisation(&mut self, name: &str, kv: &KV) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().watch_initialisation(name, kv)?;
        }
        Ok(())
    }

    fn watch_finalisation(&mut self, state: &S, kv: &KV) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().watch_finalisation(state, kv)?;
        }
        Ok(())
    }

    fn watch_iteration(&mut self, state: &S, kv: &KV) -> Result<(), WatchError> {
        for watcher in &mut self.watchers {
            let iter = state.current_iteration();
            let observer = &mut watcher.0.lock().unwrap();
            match watcher.1 {
                Frequency::Always => observer.watch_iteration(state, kv),
                Frequency::Every(i) if iter % i == 0 => observer.watch_iteration(state, kv),
                Frequency::Never | Frequency::Every(_) | Frequency::Last => Ok(()),
            }?;
        }
        Ok(())
    }

    fn finalise_watcher(&mut self, name: &str) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().finalise_watcher(name)?;
        }
        Ok(())
    }

    fn initialise_watcher(&mut self, name: &str) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().initialise_watcher(name)?;
        }
        Ok(())
    }

    fn initialise_iteration(&mut self, name: &str) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().initialise_iteration(name)?;
        }
        Ok(())
    }
}
