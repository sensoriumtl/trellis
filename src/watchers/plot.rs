use crate::kv::KV;
use crate::plotters::{PlotConfig, PlottableLine, Plotter};
use crate::state::{State, TrellisFloat};
use crate::watchers::{ObservationError, Observer, Stage};
use ndarray::{Array1, ArrayView1};
use std::cell::RefCell;
use std::path::PathBuf;

use super::Target;

pub struct PlotGenerator<R: PartialOrd> {
    plotter: RefCell<Plotter<R>>,
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
    pub fn param(
        dir: PathBuf,
        identifier: String,
        config: PlotConfig<R>,
        nodes: ArrayView1<'_, R>,
        target: Target,
    ) -> Self {
        Self {
            plotter: Plotter::new(dir, identifier, config, Some(nodes)).into(),
            target,
        }
    }

    pub fn measure(dir: PathBuf, identifier: String, config: PlotConfig<R>) -> Self {
        Self {
            plotter: Plotter::new(dir, identifier, config, None).into(),
            target: Target::Measure,
        }
    }
}

impl<S: State, R> Observer<S> for PlotGenerator<R>
where
    S: State<Float = R>,
    <S as State>::Param: Clone + Into<Array1<R>>,
    R: Clone + Default + PartialOrd + TrellisFloat + 'static,
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
impl<R> PlotGenerator<R>
where
    R: Clone + Default + PartialOrd + TrellisFloat + 'static,
{
    fn observe_iteration<S>(&self, state: &S, _kv: Option<&KV>) -> Result<(), ObservationError>
    where
        S: State<Float = R>,
        <S as State>::Param: Clone + Into<Array1<R>>,
    {
        match self.target {
            Target::Param => {
                if let Some(param) = state.get_param() {
                    let iter = state.current_iteration();
                    let item = Item {
                        identifier: format!("{iter}"),
                        data: param.clone().into(),
                    };
                    let mut plotter = self.plotter.borrow_mut();
                    plotter.plot_line(&item).unwrap();
                }
            }
            Target::Measure => {
                let iteration = state.current_iteration();
                let measure = state.measure();
                let mut plotter = self.plotter.borrow_mut();
                plotter.plot_point(iteration, measure).unwrap();
            }
        }
        Ok(())
    }
}
