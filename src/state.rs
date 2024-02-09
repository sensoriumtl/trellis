use hifitime::Duration;

#[derive(Eq, PartialEq)]
pub enum Status {
    Terminated(Reason),
    NotTerminated,
}

#[derive(Eq, PartialEq)]
pub enum Reason {
    ControlC,
    Controller,
    ExceededMaxIterations,
}

pub trait State {
    fn new() -> Self;
    fn record_time(&mut self, duration: Duration);
    fn update(&mut self);
    fn is_initialised(&self) -> bool;
    fn is_terminated(&self) -> bool;
    fn terminate_due_to(self, reason: Reason) -> Self;
}
