use serde::Serialize;
use std::path::PathBuf;

use crate::{
    watchers::{ObservationError, Observer, Stage, Subject, Target},
    writers::{WriteToFileSerializer, Writeable, Writer},
    State, KV,
};

pub struct FileWriter {
    writer: Writer,
    serializer: WriteToFileSerializer,
    target: Target,
}

struct WriteableItem<'a, P> {
    identifier: String,
    data: &'a P,
}

impl<'a, P: Serialize> Writeable for WriteableItem<'a, P> {
    type Data = P;

    fn identifier(&self) -> &'_ str {
        &self.identifier
    }

    fn data(&self) -> &'a Self::Data {
        self.data
    }
}

impl FileWriter {
    pub fn new(
        dir: PathBuf,
        identifier: String,
        serializer: WriteToFileSerializer,
        target: Target,
    ) -> Self {
        Self {
            writer: Writer::new(dir, identifier).unwrap(),
            serializer,
            target,
        }
    }

    #[must_use]
    pub(crate) fn with_writeable_identifier(mut self, identifier: String) -> Self {
        self.writer = self.writer.with_writeable_identifier(identifier);
        self
    }
}

impl<'a, S: State> Observer for FileWriter {
    type Subject = Subject<'a, S>;
    fn observe(&self, subject: &Self::Subject) {
        match subject.stage {
            Stage::Initialisation => self.observe_initialisation(subject.ident, subject.key_value),
            Stage::Finalisation => self.observe_finalisation(subject.ident, subject.key_value),
            Stage::Iteration => self.observe_iteration(subject.state, subject.key_value),
        }
    }
}

/// `WriteToFile` only implements `observer_iter` and not `observe_init` to avoid saving the
/// initial parameter vector. It will only save if there is a parameter vector available in the
/// state, otherwise it will skip saving silently.
impl<S> FileWriter
where
    S: State,
    <S as State>::Param: Serialize,
{
    fn watch_iteration(&mut self, state: &S, _kv: &KV) -> Result<(), ObservationError> {
        match self.target {
            Target::Param => {
                if let Some(param) = state.get_param() {
                    let iter = state.current_iteration();
                    let writeable = WriteableItem {
                        identifier: format!("{iter}"),
                        data: param,
                    };
                    self.writer
                        .write(self.serializer, &writeable)
                        .map_err(|e| ObservationError::Writer(Box::new(e)))?;
                }
            }
            Target::Measure => {
                let iter = state.current_iteration();
                let measure = state.measure();
                self.writer
                    .write_pair(iter, measure)
                    .map_err(|e| ObservationError::Writer(Box::new(e)))?;
            }
        }
        Ok(())
    }
}
