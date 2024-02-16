use ndarray::{s, Array1, ArrayView1, ArrayView2};
use plotly::{
    common::Title,
    layout::{themes::PLOTLY_DARK, Axis},
    Contour, Layout, Plot, Scatter,
};
use serde::Serialize;
use std::ops::Range;
use std::path::PathBuf;

use crate::state::TrellisFloat;

#[derive(Debug, thiserror::Error)]
pub enum PlotterError {
    #[error("dimensional mismatch in plot variables")]
    DimensionMismatch,
}

pub trait PlottableLine<'a, R> {
    fn independent_variable(&'a self) -> ArrayView1<'a, R>;
    fn dependent_variable(&'a self) -> Option<ArrayView1<'a, R>> {
        None
    }
    fn identifier(&'a self) -> &'a str;
}

pub trait PlottableHeatmap<'a, R> {
    fn independent_variable(&'a self) -> ArrayView1<'a, R>;
    fn dependent_variable(&'a self) -> Option<ArrayView1<'a, R>> {
        None
    }
    fn heatmap(&'a self) -> ArrayView2<'a, R>;
    fn identifier(&'a self) -> &'a str;
}

#[derive(Clone, Debug)]
pub struct PlotConfig<R> {
    pub(crate) x_limits: Range<R>,
    pub(crate) y_limits: Option<Range<R>>,
    pub(crate) x_label: String,
    pub(crate) y_label: String,
    pub(crate) title: String,
}

impl<F: TrellisFloat> PlotConfig<F> {
    fn to_layout(&self) -> Layout {
        let x_axis = Axis::new()
            .range(vec![
                format!("{}", self.x_limits.start),
                format!("{}", self.x_limits.end),
            ])
            .title(Title::new(&format!("<b>{}</b>", self.x_label)));
        let y_axis = Axis::new().title(Title::new(&format!("<b>{}</b>", self.y_label)));

        Layout::new()
            .template(&*PLOTLY_DARK)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .show_legend(true)
            .title(Title::new(&format!("<b>{}</b>", self.title)))
            .width(1000)
            .height(1000)
    }
}

pub struct Plotter<R> {
    output_path: PathBuf,
    plot: Plot,
    config: PlotConfig<R>,
    grid_points: Array1<R>,
}

impl<R> Plotter<R>
where
    R: Clone + Default + PartialOrd + Serialize + TrellisFloat + 'static,
{
    pub(crate) fn new<'a>(
        mut output_directory: PathBuf,
        filename: String,
        config: PlotConfig<R>,
        nodes: ArrayView1<'a, R>,
    ) -> Self {
        output_directory.push(format!("{filename}.html"));
        Self {
            output_path: output_directory,
            plot: Plot::new(),
            config,
            grid_points: nodes.to_owned(),
        }
    }

    pub(crate) fn plot_line<'a, P: PlottableLine<'a, R>>(
        &mut self,
        item: &'a P,
    ) -> Result<(), PlotterError> {
        let independent_variable: ArrayView1<'a, R> = item.independent_variable();
        if independent_variable.len() == self.grid_points.len() {
            let trace =
                Scatter::from_array(self.grid_points.clone(), independent_variable.to_owned())
                    .name(item.identifier());
            self.plot.add_trace(trace);
            self.plot.set_layout(self.config.to_layout());
            self.plot.write_html(&self.output_path);
            return Ok(());
        }

        Err(PlotterError::DimensionMismatch)
    }

    pub(crate) fn plot_line_internal<'a, P: PlottableLine<'a, R>>(
        &mut self,
        item: &'a P,
    ) -> Result<(), PlotterError> {
        let independent_variable: ArrayView1<'a, R> = item.independent_variable();
        if independent_variable.len() == self.grid_points.len() - 2 {
            let trace = Scatter::from_array(
                self.grid_points
                    .clone()
                    .slice_move(s![1..independent_variable.len()]),
                independent_variable.to_owned(),
            )
            .name(item.identifier());
            self.plot.add_trace(trace);
            self.plot.set_layout(self.config.to_layout());
            self.plot.write_html(&self.output_path);
            return Ok(());
        }

        Err(PlotterError::DimensionMismatch)
    }

    pub(crate) fn plot_heatmap_internal<'a, P: PlottableHeatmap<'a, R>>(
        &mut self,
        item: &'a P,
    ) -> Result<(), PlotterError> {
        let independent_variable: ArrayView1<'a, R> = item.independent_variable();
        let heatmap: ArrayView2<'a, R> = item.heatmap();
        if heatmap.shape()[0] == self.grid_points.len() - 2 {
            let mut z = vec![];
            for row in heatmap.columns() {
                z.push(row.to_vec());
            }
            let x = self
                .grid_points
                .clone()
                .slice_move(s![1..heatmap.shape()[0]])
                .to_vec();
            let y = independent_variable.to_owned().to_vec();
            let trace = Contour::new(x, y, z).name(item.identifier());
            self.plot.add_trace(trace);
            self.plot.set_layout(self.config.to_layout());
            self.plot.write_html(&self.output_path);
            return Ok(());
        }

        Err(PlotterError::DimensionMismatch)
    }
}
