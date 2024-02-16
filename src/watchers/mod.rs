use std::sync::{Arc, Mutex};

use crate::{kv::KV, writers::WriterError, State};

mod file;
pub use file::FileWriter;

mod plot;
pub use plot::PlotGenerator;

#[cfg(feature = "slog")]
mod slog;

#[cfg(feature = "slog")]
pub use slog::SlogLogger;

pub enum Target {
    Param,
    Measure,
}

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
pub(crate) enum WatchError {
    #[error("error in writer")]
    Writer(#[from] WriterError),
}

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
    fn watch_initialisation(&mut self, _name: &str, _kv: &KV) -> Result<(), WatchError> {
        Ok(())
    }

    fn watch_finalisation(&mut self, _name: &str, _kv: &KV) -> Result<(), WatchError> {
        Ok(())
    }

    fn watch_iteration(&mut self, _state: &S, _kv: &KV) -> Result<(), WatchError> {
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

    fn watch_finalisation(&mut self, name: &str, kv: &KV) -> Result<(), WatchError> {
        for watcher in &self.watchers {
            watcher.0.lock().unwrap().watch_finalisation(name, kv)?;
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
}
