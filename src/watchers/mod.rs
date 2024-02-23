use std::sync::{Arc, Mutex};

use crate::kv::KV;

#[cfg(feature = "writing")]
mod file;

#[cfg(feature = "writing")]
pub use file::FileWriter;

#[cfg(feature = "plotting")]
mod plot;
#[cfg(feature = "plotting")]
pub use plot::PlotGenerator;

#[cfg(feature = "slog")]
mod slog;

#[cfg(feature = "slog")]
pub use slog::SlogLogger;

mod tracing;
pub use tracing::Tracer;

pub enum Target {
    Param,
    Measure,
}

#[derive(Copy, Clone)]
pub enum Stage {
    Initialisation,
    Finalisation,
    Iteration,
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub(crate) struct ObserverVec<S>(Vec<(Arc<Mutex<dyn Observer<S>>>, Frequency)>);

impl<S> ObserverVec<S> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl<S> Default for ObserverVec<S> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<S> ObserverVec<S> {
    pub(crate) fn as_slice(&self) -> ObserverSlice<'_, S> {
        ObserverSlice(&self.0[..])
    }
}

#[allow(clippy::type_complexity)]
pub(crate) struct ObserverSlice<'a, S>(&'a [(Arc<Mutex<dyn Observer<S>>>, Frequency)]);

pub trait Observer<S> {
    fn observe(&self, ident: &'static str, subject: &S, kv: Option<&KV>, stage: Stage);
}

pub trait Observable<S> {
    type Observer;
    fn update(&self, ident: &'static str, subject: &S, kv: Option<&KV>, stage: Stage);
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency);
    fn detach(&mut self, observer: Self::Observer);
}

#[derive(Clone)]
pub(crate) struct Subject<D> {
    pub(crate) data: D,
    pub(crate) observers: ObserverVec<D>,
}

impl<S> Observable<S> for ObserverVec<S> {
    type Observer = Arc<Mutex<dyn Observer<S>>>;
    fn update(&self, ident: &'static str, subject: &S, kv: Option<&KV>, stage: Stage) {
        self.0
            .iter()
            .map(|o| o.0.lock().unwrap())
            .for_each(|o| o.observe(ident, subject, kv, stage));
    }
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency) {
        self.0.push((observer, frequency));
    }
    fn detach(&mut self, observer: Self::Observer) {
        self.0.retain(|f| !Arc::ptr_eq(&f.0, &observer));
    }
}

// pub trait Observable<'a>
// where
//     Self: 'a,
// {
//     type Observer<'a>;
//     fn update(&self);
//     fn attach<'a>(&mut self, observer: Self::Observer<'a>, frequency: Frequency);
//     fn detach<'a>(&mut self, observer: Self::Observer<'a>);
// }
//
// impl<C, P, S, R> Observable for Runner<C, P, S, R> {
//     type Observer<'a> = Arc<dyn Observer<Subject = Subject<'a, S>>>;
//     fn update(&self) {
//         self.observers().update()
//     }
//     fn attach<'a>(&mut self, observer: Self::Observer<'a>, frequency: Frequency) {
//         self.observers_mut().attach(observer, frequency);
//     }
//     fn detach<'a>(&mut self, observer: Self::Observer<'a>) {
//         self.observers_mut().retain(observer)
//     }
// }
//
// impl<S> Observable for ObserverVec<S> {
//     type Observer<'a> = Arc<dyn Observer<Subject = Subject<'a, S>>>;
//     fn update(&self) {
//         self.0
//             .iter()
//             .flat_map(|o| o.upgrade())
//             .for_each(|o| o.observe(self));
//     }
//     fn attach<'a>(&mut self, observer: Self::Observer<'a>, frequency: Frequency) {
//         self.0.push((Arc::downgrade(&observer), frequency));
//     }
//     fn detach<'a>(&mut self, observer: Self::Observer<'a>) {
//         self.0.retain(|f| !f.0.ptr_eq(&Arc::downgrade(&observer)));
//     }
// }

#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    #[error("error in writer")]
    Writer(Box<dyn std::error::Error + 'static>), // We don't wrap the actual error, as we don't want to import the deps unless requested
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
