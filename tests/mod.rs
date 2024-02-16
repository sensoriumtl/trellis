use std::{fs::File, sync::Arc};

use hifitime::Duration;
use runner::prelude::*;
use tracing::Level;
use tracing_subscriber::{filter, prelude::*};

fn init() {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // A layer that logs events to a file.
    let file = File::create("debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));

    // A layer that collects metrics using specific events.
    let metrics_layer = filter::LevelFilter::INFO;

    tracing_subscriber::registry()
        .with(
            stdout_log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO)
                // Combine the filtered `stdout_log` layer with the
                // `debug_log` layer, producing a new `Layered` layer.
                .and_then(debug_log)
                // Add a filter to *both* layers that rejects spans and
                // events whose targets start with `metrics`.
                .with_filter(filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("metrics")
                })),
        )
        .with(
            // Add a filter to the metrics label that *only* enables
            // events whose targets start with `metrics`.
            metrics_layer.with_filter(filter::filter_fn(|metadata| {
                metadata.target().starts_with("metrics")
            })),
        )
        .init();
}

struct DummyCalculation {}

struct DummyProblem {}

struct DummyState {
    iteration: usize,
    is_initialised: bool,
    termination_status: Status,
    time_elapsed: Option<Duration>,
}

impl State for DummyState {
    fn new() -> Self {
        Self {
            time_elapsed: None,
            iteration: 0,
            is_initialised: false,
            termination_status: Status::NotTerminated,
        }
    }

    fn record_time(&mut self, duration: Duration) {
        self.time_elapsed = Some(duration);
    }

    fn increment_iteration(&mut self) {
        self.iteration += 1;
    }

    fn current_iteration(&self) -> usize {
        self.iteration
    }

    fn update(&mut self) {}

    fn is_initialised(&self) -> bool {
        self.is_initialised
    }

    fn is_terminated(&self) -> bool {
        self.termination_status != Status::NotTerminated
    }

    fn terminate_due_to(mut self, reason: Reason) -> Self {
        self.termination_status = Status::Terminated(reason);
        self
    }
}

#[derive(Debug)]
enum DummyError {
    TypeA,
}

impl std::fmt::Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DummyError {}

impl Calculation<DummyProblem, DummyState> for DummyCalculation {
    type Error = DummyError;
    const NAME: &'static str = "My dumb calculation";
    fn initialise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<(DummyState, Option<KV>), Self::Error> {
        println!("initialising");
        Ok((state, None))
    }

    fn next(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        mut state: DummyState,
    ) -> Result<(DummyState, Option<KV>), Self::Error> {
        println!("taking step {}", state.iteration);

        if state.iteration >= 100 {
            state = state.terminate_due_to(Reason::ExceededMaxIterations);
        }

        Ok((state, None))
    }

    fn finalise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<(DummyState, Option<KV>), Self::Error> {
        println!("finalising");
        Ok((state, None))
    }
}

#[test]
fn problems_run_successfully() {
    init();
    let calculation = DummyCalculation {};
    let problem = DummyProblem {};

    let runner = calculation
        .build_for(problem)
        .with_watcher(TracingLogger::new(Level::INFO), Frequency::Always)
        .finalise()
        .expect("failed to build problem");

    let result = runner.run();
}
