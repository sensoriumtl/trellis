use serde::Serialize;
use std::path::PathBuf;

use crate::{
    writers::{WriteToFileSerializer, Writeable, Writer},
    State, KV,
};

use super::{Target, Watch, WatchError};

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

/// `WriteToFile` only implements `observer_iter` and not `observe_init` to avoid saving the
/// initial parameter vector. It will only save if there is a parameter vector available in the
/// state, otherwise it will skip saving silently.
impl<S> Watch<S> for FileWriter
where
    S: State,
    <S as State>::Param: Serialize,
{
    fn watch_iteration(&mut self, state: &S, _kv: &KV) -> Result<(), WatchError> {
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
                        .map_err(|e| WatchError::Writer(Box::new(e)))?;
                }
            }
            Target::Measure => {
                let iter = state.current_iteration();
                let measure = state.measure();
                self.writer
                    .write_pair(iter, measure)
                    .map_err(|e| WatchError::Writer(Box::new(e)))?;
            }
        }
        Ok(())
    }
}
