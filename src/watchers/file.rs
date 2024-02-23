use serde::Serialize;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::{
    watchers::{ObservationError, Observer, Stage, Target},
    writers::{WriteToFileSerializer, Writeable, Writer},
    State, KV,
};

pub struct FileWriter {
    writer: RefCell<Writer>,
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
            writer: RefCell::new(Writer::new(dir, identifier).unwrap()),
            serializer,
            target,
        }
    }

    #[must_use]
    pub(crate) fn with_writeable_identifier(self, identifier: String) -> Self {
        self.writer
            .borrow_mut()
            .with_writeable_identifier(identifier);
        self
    }
}

impl<S> Observer<S> for FileWriter
where
    S: State,
    <S as State>::Param: Serialize,
{
    fn observe(&self, _ident: &'static str, subject: &S, key_value: Option<&KV>, stage: Stage) {
        match stage {
            Stage::Iteration => self.observe_iteration(subject, key_value),
            _ => Ok(()),
        }
        .unwrap()
    }
}

/// `WriteToFile` only implements `observer_iter` and not `observe_init` to avoid saving the
/// initial parameter vector. It will only save if there is a parameter vector available in the
/// state, otherwise it will skip saving silently.
impl FileWriter {
    fn observe_iteration<S>(&self, state: &S, _kv: Option<&KV>) -> Result<(), ObservationError>
    where
        S: State,
        <S as State>::Param: Serialize,
    {
        match self.target {
            Target::Param => {
                if let Some(param) = state.get_param() {
                    let iter = state.current_iteration();
                    let writeable = WriteableItem {
                        identifier: format!("{iter}"),
                        data: param,
                    };
                    let mut writer = self.writer.borrow_mut();
                    writer
                        .write(self.serializer, &writeable)
                        .map_err(|e| ObservationError::Writer(Box::new(e)))?;
                }
            }
            Target::Measure => {
                let iter = state.current_iteration();
                let measure = state.measure();
                let mut writer = self.writer.borrow_mut();
                writer
                    .write_pair(iter, measure)
                    .map_err(|e| ObservationError::Writer(Box::new(e)))?;
            }
        }
        Ok(())
    }
}
