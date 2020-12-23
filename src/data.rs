#![allow(dead_code)]

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Index;

use gnuplot::AutoOption::Fix;
use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};
use rand::thread_rng;
use rand_distr::num_traits::{AsPrimitive, Pow};
use rand_distr::{Distribution, Normal};

use crate::strategy::{GenResult, QueryCalculationDto};
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};

pub type Symbol = String;

#[derive(Debug, Clone)]
pub struct AssetNotFoundError {
    symbol: Symbol,
}

impl AssetNotFoundError {
    pub fn new(symbol: Symbol) -> Self {
        AssetNotFoundError { symbol }
    }
}

impl fmt::Display for AssetNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "asset with symbol {} not found\n", self.symbol)
    }
}

impl std::error::Error for AssetNotFoundError {
    fn description(&self) -> &str {
        "Asset not found."
    }
}

// TODO query memoization/caching
pub trait DataClient {
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset>;
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query: Option<QueryCalculationDto>,
    ) -> GenResult<&TimeSeries1D>;
}

#[derive(Debug, Eq, PartialEq, Hash)]
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
            ts_vec[i].index().clone(),
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
#[cfg(test)]
mod tests {
    // silence approx lib warnings
    #![allow(unused_must_use)]

    use std::collections::HashSet;

    use rand_distr::num_traits::AsPrimitive;

    use crate::data::*;
    use crate::simulation::{MockDataClient, DATA_SIZE, TODAY};

    const EPSILON: f64 = 1E-10;

    #[test]
    fn mock_data_client_assets() {
        let client = MockDataClient::new();
        // println!("{:?}", client);
        let symbols: HashSet<&Symbol> = client.assets().keys().collect();
        // println!("{:?}", symbols);
        assert_eq!(
            symbols,
            vec![Symbol::from("A"), Symbol::from("B")].iter().collect()
        )
    }

    #[test]
    fn mock_data_client_query() {
        let client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        // println!("{:?}", client);
        let asset = Asset::new(Symbol::from("A"));
        let ts = client.query(&asset, &TODAY, None).unwrap();
        // println!("{:?}", ts);
        assert_eq!(ts.len(), DATA_SIZE);
    }
}
