#![allow(dead_code)]

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Index;

use chrono::{DateTime, Utc};
use gnuplot::AutoOption::Fix;
use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};

use crate::dto::strategy::QueryCalculationDto;
use crate::errors::GenResult;
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use std::borrow::Borrow;

pub type Symbol = String;

// TODO query memoization/caching
pub trait DataClient {
    fn assets(&self) -> &HashMap<Symbol, Asset>;
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset>;
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query: Option<QueryCalculationDto>,
    ) -> GenResult<&TimeSeries1D>;
}
impl fmt::Debug for dyn DataClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataClient")
            .field("assets", &self.assets().len())
            .finish()
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Asset {
    symbol: Symbol,
}

impl Asset {
    pub fn new(symbol: Symbol) -> Self {
        Asset { symbol }
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
pub fn plot_ts_values(ts_vec: Vec<TimeSeries1D>) {
    plot_ts(ts_vec.iter().collect())
}
pub fn plot_ts(ts_vec: Vec<&TimeSeries1D>) {
    let mut fg = Figure::new();
    let ys: Vec<&Vec<f64>> = ts_vec.iter().map(|ts| ts.values()).collect();
    let y_max: f64 = ys
        .as_slice()
        .iter()
        .flat_map(|x| x.iter())
        .fold(f64::NAN, |a, b| f64::max(a, *b).clone())
        * 1.1;
    let mut y_min = ys
        .as_slice()
        .iter()
        .flat_map(|x| x.iter())
        .fold(f64::NAN, |a, b| f64::min(a, *b).clone());
    // handle negative y-values
    y_min = f64::min(y_min - y_min.abs() * 0.1, 0f64);
    let xs: Vec<&Vec<DateTime<Utc>>> = ts_vec.iter().map(|ts| ts.index()).collect();
    let x_max: DateTime<Utc> = xs
        .iter()
        .flat_map(|x| x.iter())
        .fold(chrono::MIN_DATETIME, |a, b| DateTime::max(a, *b).clone());
    let x_min: DateTime<Utc> = xs
        .iter()
        .flat_map(|x| x.iter())
        .fold(chrono::MAX_DATETIME, |a, b| DateTime::min(a, *b).clone());
    let x_label = format!("{:?} - {:?}", x_min, x_max);
    let axis = fg
        .axes2d()
        .set_title("Time Series Plot", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("index", &[])
        .set_x2_label(x_label.as_str(), &[])
        .set_y_label("values", &[])
        .set_y_range(Fix::<f64>(y_min), Fix::<f64>(y_max));
    for i in 0..ys.len() {
        axis.lines(
            0..ts_vec[i].len(),
            ts_vec[i].values().clone(),
            &[Caption(&format!("{}", i))],
        );
    }
    fg.show().unwrap();
}

pub fn plots(x: Vec<f64>, ys: Vec<&Vec<f64>>) {
    let mut fg = Figure::new();
    let y_max: f64 = ys
        .as_slice()
        .iter()
        .flat_map(|x| x.iter())
        .fold(f64::NAN, |a, b| f64::max(a, *b).clone())
        * 1.1;
    let mut y_min = ys
        .as_slice()
        .iter()
        .flat_map(|x| x.iter())
        .fold(f64::NAN, |a, b| f64::min(a, *b).clone());
    y_min = f64::min(y_min - y_min.abs() * 0.1, 0f64);
    let axis = fg
        .axes2d()
        .set_title("Time Series Plot", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("timestamp", &[])
        .set_y_label("values", &[])
        .set_y_range(Fix::<f64>(y_min), Fix::<f64>(y_max));
    for i in 0..ys.len() {
        axis.lines(
            x.clone(),
            ys.get(i).unwrap().clone(),
            &[Caption(&format!("{}", i))],
        );
    }
    fg.show().unwrap();
}

pub fn plot(x: &Vec<f64>, y: &Vec<f64>) {
    let mut fg = Figure::new();
    let y_max: f64 = y
        .as_slice()
        .iter()
        .fold(f64::NAN, |a, b| f64::max(a, *b).clone())
        * 1.1;
    let mut y_min = y
        .as_slice()
        .iter()
        .fold(f64::NAN, |a, b| f64::min(a, *b).clone());
    y_min = f64::min(y_min - y_min.abs() * 0.1, 0f64);
    fg.axes2d()
        .set_title("Time Series Plot", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("timestamp", &[])
        .set_y_label("values", &[])
        .set_y_range(Fix::<f64>(y_min), Fix::<f64>(y_max))
        .lines(x.clone(), y.clone(), &[Caption("Price")]);
    fg.show().unwrap();
}
