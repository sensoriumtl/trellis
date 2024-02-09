use hifitime::Duration;
use runner::prelude::*;

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

    fn update(&mut self) {
        self.iteration += 1;
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
    fn initialise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<DummyState, Self::Error> {
        println!("initialising");
        Ok(state)
    }

    fn next(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        mut state: DummyState,
    ) -> Result<DummyState, Self::Error> {
        println!("taking step {}", state.iteration);

        if state.iteration >= 100 {
            state = state.terminate_due_to(Reason::ExceededMaxIterations);
        }

        Ok(state)
    }

    fn finalise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<DummyState, Self::Error> {
        println!("finalising");
        Ok(state)
    }
}

#[test]
fn problems_run_successfully() {
    let calculation = DummyCalculation {};
    let problem = DummyProblem {};

    let runner = calculation
        .build_for(problem)
        .finalise()
        .expect("failed to build problem");

    let result = runner.run();
}
