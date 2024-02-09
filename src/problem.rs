pub struct Problem<P>(P);

impl<P> Problem<P> {
    pub(crate) fn new(inner: P) -> Self {
        Self(inner)
    }
}
