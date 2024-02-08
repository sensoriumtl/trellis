use hifitime::Duration;

pub enum Reason {
    ControlC,
    Controller,
}

pub trait State {
    fn new() -> Self;
    fn record_time(&mut self, duration: Duration);
    fn update(&mut self);
    fn is_initialised(&self) -> bool;
    fn terminate_due_to(self, reason: Reason) -> Self;
}
