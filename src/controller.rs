//! Controllers are external processes which can kill the main loop.

use std::thread;

#[cfg(feature = "tokio")]
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

/// A controller has to implement the `Control` trait
pub trait Control: Send {
    /// Value emitted by the controller when the kill signal is received
    type Value;
    type Error;
    /// Spawns a blocking task that waits for a kill signal to be returned
    fn blocking_recv_kill_signal(self) -> Result<Self::Value, Self::Error>;
}

// Set up a handler for the kill signal
//
// This function spawns a background thread which awaits on the blocking_recv call for the kill
// signal. When a `Value` or `Error` is received from the receiver the provided closure is called
pub(crate) fn set_handler<R, F>(
    receiver: R,
    // When the kill signal is received, this closure is called
    mut handle_kill_signal: F,
) -> Result<(), std::io::Error>
where
    R: Control + 'static,
    F: FnMut() + 'static + Send,
{
    thread::Builder::new()
        .name("kill_signal".into())
        .spawn(move || {
            let _ = receiver.blocking_recv_kill_signal();
            handle_kill_signal()
        })?;
    Ok(())
}

#[cfg(feature = "tokio")]
impl Control for CancellationToken {
    type Value = WaitForCancellationFutureOwned;
    type Error = ();
    fn blocking_recv_kill_signal(self) -> Result<Self::Value, Self::Error> {
        Ok(self.cancelled_owned())
    }
}
