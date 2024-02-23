use crate::runner::Runner;
use std::{sync::Arc, sync::Weak};

use crate::kv::KV;

// #[cfg(feature = "writing")]
// mod file;
//
// #[cfg(feature = "writing")]
// use crate::writers::WriterError;
//
// #[cfg(feature = "writing")]
// pub use file::FileWriter;
//
// #[cfg(feature = "plotting")]
// mod plot;
// #[cfg(feature = "plotting")]
// pub use plot::PlotGenerator;
//
// #[cfg(feature = "slog")]
// mod slog;
//
// #[cfg(feature = "slog")]
// pub use slog::SlogLogger;
//
// mod tracing;
// pub use tracing::Tracer;

pub enum Target {
    Param,
    Measure,
}

#[derive(Copy, Clone)]
pub(crate) enum Stage {
    Initialisation,
    Finalisation,
    Iteration,
}

#[derive(Clone)]
pub(crate) struct ObservationData<'a, S> {
    pub(crate) ident: &'static str,
    pub(crate) kv: Option<&'a KV>,
    pub(crate) state: &'a S,
    pub(crate) stage: Stage,
}

// impl<'a, S: Clone> ObserverVec<Subject<'a, S>> {
//     pub(crate) fn observe_initialisation(
//         &'a self,
//         ident: &'static str,
//         state: &'a S,
//         kv: Option<&'a KV>,
//     ) {
//         let subject = Subject {
//             ident,
//             kv,
//             state,
//             observers: self.clone(),
//             stage: Stage::Initialisation,
//         };
//         todo!()
//     }
//
//     pub(crate) fn finalisation_subject(
//         &'a self,
//         ident: &'static str,
//         state: &'a S,
//     kv: Option<&'a KV>,
// ) -> Subject<'a, S> {
//     Subject {
//         ident,
//         kv,
//         state,
//         observers: self.clone(),
//         stage: Stage::Finalisation,
//     }
// }
//
// pub(crate) fn iteration_subject(
//     &'a self,
//     ident: &'static str,
//     state: &'a S,
//     kv: Option<&'a KV>,
//     ) -> Subject<'a, S> {
//         Subject {
//             ident,
//             kv,
//             state,
//             observers: self.clone(),
//             stage: Stage::Iteration,
//         }
//     }
// }

#[derive(Clone, Default)]
pub(crate) struct ObserverVec<S>(Vec<(Weak<dyn Observer<S>>, Frequency)>);

impl<S> ObserverVec<S> {
    pub(crate) fn as_slice<'a>(&'a self) -> ObserverSlice<'a, S> {
        ObserverSlice(&self.0[..])
    }
}

pub(crate) struct ObserverSlice<'a, S>(&'a [(Weak<dyn Observer<S>>, Frequency)]);

pub trait Observer<S> {
    fn observe(&self, subject: &S);
}

pub trait Observable<S> {
    type Observer;
    fn update(&self, subject: &S);
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency);
    fn detach(&mut self, observer: Self::Observer);
}

#[derive(Clone)]
pub(crate) struct Subject<D> {
    pub(crate) data: D,
    pub(crate) observers: ObserverVec<D>,
}

impl<S> Observable<S> for ObserverVec<S> {
    type Observer = Arc<dyn Observer<S>>;
    fn update(&self, subject: &S) {
        self.0
            .iter()
            .flat_map(|o| o.0.upgrade())
            .for_each(|o| o.observe(subject));
    }
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency) {
        self.0.push((Arc::downgrade(&observer), frequency));
    }
    fn detach(&mut self, observer: Self::Observer) {
        self.0.retain(|f| !f.0.ptr_eq(&Arc::downgrade(&observer)));
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
