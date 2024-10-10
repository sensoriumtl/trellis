use crate::Cause;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Copy, Clone)]
#[non_exhaustive]
// Holder of a killswitch
pub(super) enum Holder {
    // A killswitch which can be activated using Ctrl-c in the terminal
    CtrlC,
    // A controlling parent process
    Parent,
}

impl From<Holder> for Cause {
    fn from(val: Holder) -> Self {
        match val {
            Holder::CtrlC => Cause::ControlC,
            Holder::Parent => Cause::Parent,
        }
    }
}

// A killswitch is able to terminate a running calculation
//
// Killswitches are checked during each iteration and if any are activated the runner is terminated
pub(super) struct Killswitch {
    // The type of holder which holds the killswitch
    holder: Holder,
    // Holds the status of the killswitch
    //
    // When the inner value is `true` the killswitch has been activated
    inner: Arc<AtomicBool>,
}

impl Killswitch {
    pub(super) fn new(holder: Holder, inner: Arc<AtomicBool>) -> Self {
        Self { holder, inner }
    }
    // Returns true if the killswitch has been activated
    pub(super) fn is_dead(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }

    pub(super) fn held_by(&self) -> Holder {
        self.holder
    }
}
