#![allow(dead_code)]

mod calculation;
mod controller;

// #[cfg(feature = "plotting")]
// mod plotters;

pub mod prelude;
mod problem;
mod result;
mod runner;
mod state;
mod watchers;

mod staten;

// #[cfg(feature = "writing")]
// mod writers;

pub use calculation::Calculation;
pub(crate) use controller::Control;

// #[cfg(feature = "plotting")]
// pub use plotters::PlotConfig;
// #[cfg(feature = "plotting")]
// pub use watchers::PlotGenerator;

pub use problem::Problem;
pub use result::Output;
pub use runner::GenerateBuilder;
pub use state::{Reason, State, Status};
pub use watchers::Tracer;
pub use watchers::{Frequency, Target};

// #[cfg(feature = "writing")]
// pub use watchers::FileWriter;

// #[cfg(feature = "writing")]
// pub use writers::WriteToFileSerializer;

pub use hifitime::Duration;

pub use state::TrellisFloat;
