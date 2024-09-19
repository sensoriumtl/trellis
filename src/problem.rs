#[derive(Debug)]
pub struct Problem<P>(P);

impl<P> Problem<P> {
    pub(crate) fn new(inner: P) -> Self {
        Self(inner)
    }
}

impl<P> AsRef<P> for Problem<P> {
    fn as_ref(&self) -> &P {
        &self.0
    }
}
