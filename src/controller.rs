//! Controllers are external processes which can kill the main loop.

use std::thread;

/// A controller has to implement the `Control` trait
pub trait Control: Send {
    type Value;
    type Error;
    fn blocking_recv_kill_signal(&self) -> Result<Self::Value, Self::Error>;
}

pub(crate) fn set_handler<R, F>(
    receiver: R,
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

#[cfg(tokio)]
impl<M> Control for tokio::sync::oneshot::Receiver<M> {
    type Value = M;
    type Error = tokio::sync::oneshot::error::RecvError;
    fn blocking_recv_kill_signal(&self) -> Result<Self::Value, Self::Error> {
        self.clone().blocking_recv()
    }
}
