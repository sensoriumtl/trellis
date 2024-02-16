use crate::kv::KV;
use crate::plotters::{PlotConfig, PlottableLine, Plotter};
use crate::state::{State, TrellisFloat};
use crate::watchers::Watch;
use ndarray::{Array1, ArrayView1};
use std::path::PathBuf;

use super::Target;

pub struct PlotGenerator<R: PartialOrd> {
    plotter: Plotter<R>,
    target: Target,
}

struct Item<R> {
    identifier: String,
    data: Array1<R>,
}

impl<'a, R> PlottableLine<'a, R> for Item<R> {
    fn identifier(&'a self) -> &'a str {
        &self.identifier
    }

    fn independent_variable(&'a self) -> ArrayView1<'a, R> {
        self.data.view()
    }
}

impl<R> PlotGenerator<R>
where
    R: Clone + Default + PartialOrd + TrellisFloat + 'static,
{
    pub(crate) fn new<'a>(
        dir: PathBuf,
        identifier: String,
        config: PlotConfig<R>,
        nodes: ArrayView1<'a, R>,
        target: Target,
    ) -> Self {
        Self {
            plotter: Plotter::new(dir, identifier, config, nodes),
            target,
        }
    }
}

/// `WriteToFile` only implements `observer_iter` and not `observe_init` to avoid saving the
/// initial parameter vector. It will only save if there is a parameter vector available in the
/// state, otherwise it will skip saving silently.
impl<I, R> Watch<I> for PlotGenerator<R>
where
    I: State,
    <I as State>::Param: Clone + Into<Array1<R>>,
    R: Clone + Default + PartialOrd + TrellisFloat + 'static,
{
    fn watch_iteration(&mut self, state: &I, kv: &KV) -> Result<(), super::WatchError> {
        match self.target {
            Target::Param => {
                if let Some(param) = state.get_param() {
                    let iter = state.current_iteration();
                    let item = Item {
                        identifier: format!("{iter}"),
                        data: param.clone().into(),
                    };
                    self.plotter.plot_line(&item).unwrap();
                }
            }
            Target::Measure => unimplemented!("no scatter impl"),
        }
        Ok(())
    }
}
