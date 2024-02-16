use std::{fs::File, path::PathBuf, sync::Arc};

use hifitime::Duration;
use runner::prelude::*;
use slog::Level;

struct DummyCalculation {}

struct DummyProblem {}

struct DummyState {
    iteration: usize,
    best_cost_iteration: usize,
    is_initialised: bool,
    termination_status: Status,
    time_elapsed: Option<Duration>,
    cost: f64,
    best_cost: f64,
    param: Option<Vec<f64>>,
}

impl State for DummyState {
    type Float = f64;
    type Param = Vec<f64>;
    fn new() -> Self {
        Self {
            cost: std::f64::MAX,
            best_cost: std::f64::MAX,
            param: None,
            time_elapsed: None,
            iteration: 0,
            best_cost_iteration: 0,
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

    fn update(&mut self) {
        if self.best_cost > self.cost {
            self.best_cost = self.cost;
            self.best_cost_iteration = self.iteration;
        }
    }

    fn measure(&self) -> Self::Float {
        self.cost
    }

    fn best_measure(&self) -> Self::Float {
        self.best_cost
    }

    fn iterations_since_best(&self) -> usize {
        self.iteration - self.best_cost_iteration
    }

    fn get_param(&self) -> Option<&Self::Param> {
        self.param.as_ref()
    }

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
        std::thread::sleep(std::time::Duration::from_millis(10));

        if state.iteration >= 100 {
            state = state.terminate_due_to(Reason::ExceededMaxIterations);
        }

        state.cost = (-((state.iteration as f64) / 100.0)).exp();

        Ok((state, None))
    }

    fn finalise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<(DummyState, Option<KV>), Self::Error> {
        Ok((state, None))
    }
}

#[test]
fn problems_run_successfully() {
    let calculation = DummyCalculation {};
    let problem = DummyProblem {};

    let iden = "calculation_time".to_string();
    let outdir = PathBuf::from(r"/Users/cgubbin/sensorium/tooling/runner/out/");

    let runner = calculation
        .build_for(problem)
        .with_watcher(SlogLogger::terminal(Level::Info), Frequency::Always)
        .with_watcher(
            FileWriter::new(outdir, iden, WriteToFileSerializer::JSON, Target::Measure),
            Frequency::Always,
        )
        .finalise()
        .expect("failed to build problem");

    let result = runner.run();
}
