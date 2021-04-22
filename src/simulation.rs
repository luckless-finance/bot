use std::collections::HashMap;
use std::f64::consts::PI;
use std::io::{Error, ErrorKind};

use chrono::prelude::*;

use crate::data::{Asset, DataClient, Symbol};
use crate::dto::strategy::QueryCalculationDto;
use crate::errors::{AssetNotFoundError, GenResult};
use crate::time_series::{TimeSeries1D, TimeStamp};

pub static DATA_SIZE: usize = 900;

#[derive(Debug)]
pub struct MockDataClient {
    assets: HashMap<Symbol, Asset>,
    data: HashMap<Symbol, TimeSeries1D>,
}

impl MockDataClient {
    pub fn today() -> DateTime<Utc> {
        TimeSeries1D::epoch() + TimeSeries1D::index_unit() * (DATA_SIZE - 1) as i32
    }
}

fn simulate_time_series(n: usize) -> TimeSeries1D {
    let (x0, xf) = (0f64, 6f64 * PI);
    let x = linspace(n, x0, xf);
    let amplitude = 0.5f64;
    let y0 = 10f64;
    TimeSeries1D::sin(&x, amplitude).vertical_shift(y0)
}

impl DataClient for MockDataClient {
    fn assets(&self) -> &HashMap<Symbol, Asset> {
        &self.assets
    }

    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset> {
        match self.assets.get(symbol) {
            Some(asset) => Ok(asset),
            None => Err(AssetNotFoundError::new(symbol.clone())),
        }
    }

    #[allow(unused_variables)]
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query_dto: Option<QueryCalculationDto>,
    ) -> GenResult<TimeSeries1D> {
        match self.data.get(&asset.symbol().to_string()) {
            Some(ts) => Ok(ts.filter_le(timestamp)),
            None => Err(Box::new(Error::new(ErrorKind::NotFound, "Asset not found"))),
        }
    }
}

impl MockDataClient {
    /// Create 3 `Asset` mock market
    pub fn new() -> Self {
        // A
        let (a_x0, a_xf) = (0f64, 6f64 * PI);
        let a_x = linspace(DATA_SIZE, a_x0, a_xf);
        let amplitude = 0.5f64;
        let a_y0 = 10f64;
        let a_y = TimeSeries1D::sin(&a_x, amplitude).vertical_shift(a_y0);
        // B
        let (b_x0, b_xf) = (PI, 7f64 * PI);
        let b_x = linspace(DATA_SIZE, b_x0, b_xf);
        let amplitude = 0.5f64;
        let b_y0 = 5f64;
        let b_y = TimeSeries1D::sin(&b_x, amplitude).vertical_shift(b_y0);
        // C
        let (c_x0, c_xf) = (PI, 7f64 * PI);
        let c_x = linspace(DATA_SIZE, c_x0, c_xf);
        let c_slope = 0.1f64;
        let c_y0 = 5f64;
        let c_y = TimeSeries1D::polynomial(&c_x, vec![c_y0, c_slope]);

        let data: HashMap<Symbol, TimeSeries1D> = vec![
            (Symbol::from("A"), a_y),
            (Symbol::from("B"), b_y),
            (Symbol::from("C"), c_y),
        ]
        .into_iter()
        .collect();
        let assets: HashMap<Symbol, Asset> = data
            .keys()
            .map(|x| (x.clone(), Asset::new(x.clone())))
            .into_iter()
            .collect();
        MockDataClient { assets, data }
    }
    pub fn assets(&self) -> &HashMap<Symbol, Asset> {
        &self.assets
    }
}

/// Creates vector of `n` `f64` elements monotonically increasing from `lower_bound` to `upper_bound` inclusive
#[allow(dead_code)]
fn linspace(n: usize, lower_bound: f64, upper_bound: f64) -> Vec<f64> {
    let dx = (upper_bound - lower_bound) / (n - 1) as f64;
    (0..n)
        .map(|x| x as f64 * dx)
        .map(|x| x + lower_bound)
        .collect()
}

/// `TimeSeries` generators for testing
trait TimeSeriesGenerators {
    fn polynomial(x: &Vec<f64>, coefficients: Vec<f64>) -> Self;
    fn sin(x: &Vec<f64>, amplitude: f64) -> Self;
    fn exp(x: &Vec<f64>) -> Self;
    fn add_sin(&self, periods: usize, amplitude: usize) -> Self;
    fn vertical_shift(&self, delta: f64) -> Self;
}

impl TimeSeriesGenerators for TimeSeries1D {
    /// Create new `TimeSeries` with polynomial values.
    ///
    /// Horizontal line: if `coefficients = vec![10f64]`
    /// then `y = vec![10f64, 10f64, 10f64, ...]`
    ///
    /// Flat sloped line: if `coefficients = vec![b, m]`
    /// then `y = m*x + b`
    ///
    /// In general, if (a,b,c, ...) = coefficients then,
    /// `y = a*x^0 + b*x^1 + c*x^2 + d*x^3`
    /// `  = a     + b*x   + c*x^2 + d*x^3`
    fn polynomial(x: &Vec<f64>, coefficients: Vec<f64>) -> Self {
        fn _polynomial(xi: &f64, coefficients: &[f64]) -> f64 {
            let mut sum = 0f64;
            for i in 0..coefficients.len() {
                sum = sum + coefficients[i] * xi.powf(i as f64);
            }
            sum
        }
        TimeSeries1D::from_values(
            x.iter()
                .map(|x| _polynomial(&x, coefficients.as_slice()))
                .collect(),
        )
    }
    /// Create new `TimeSeries` with exponential values.
    fn exp(x: &Vec<f64>) -> Self {
        TimeSeries1D::from_values(x.iter().map(|x| x.exp()).collect())
    }

    /// Create new `TimeSeries` with `sin` values.
    fn sin(x: &Vec<f64>, amplitude: f64) -> Self {
        TimeSeries1D::from_values(x.iter().map(|x| x.sin() * amplitude).collect())
    }

    /// Transform `TimeSeries` by adding `sin` values.
    fn add_sin(&self, periods: usize, amplitude: usize) -> Self {
        let dx = (periods as f64 * 2f64 * PI) / (self.len() - 1) as f64;
        let x: Vec<f64> = (0..self.len()).map(|xi| xi as f64 * dx).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| xi.sin() * amplitude as f64)
            .zip(self.values())
            .map(|(y1, y2)| y1 + y2)
            .collect();
        TimeSeries1D::from_vec(self.index().clone(), y)
    }

    /// Shift `TimeSeries` vertically by `delta`.
    fn vertical_shift(&self, delta: f64) -> Self {
        TimeSeries1D::from_vec(
            self.index().clone(),
            self.values().iter().map(|v| v + delta).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::f64::consts::PI;

    use crate::simulation::*;
    use crate::time_series::TimeSeries1D;

    const EPSILON: f64 = 1E-3;

    #[test]
    fn linspace_0_10() -> GenResult<()> {
        let ten = linspace(10, 0., 10.);
        assert_eq!(ten.len(), 10);
        assert_eq!(ten[0], 0f64);
        assert_eq!(ten[9], 10f64);
        Ok(())
    }

    #[test]
    fn linspace_0_2pi() -> GenResult<()> {
        let x = linspace(100, 0., PI * 2f64);
        assert_eq!(x.len(), 100);
        assert_eq!(x[0], 0f64);
        assert_eq!(x[99], PI * 2f64);
        Ok(())
    }

    #[test]
    fn sin() {
        let n: usize = 101;
        let (x0, xf) = (0f64, 2f64 * PI);
        let x = linspace(n, x0, xf);
        let amplitude = 1f64;
        let y = TimeSeries1D::sin(&x, amplitude);
        // println!("{:?}", y);
        assert_eq!(y.len(), n);
        assert_eq!(y.values()[0], 0f64);
        assert_abs_diff_eq!(y.values()[(n - 1) / 2], 0f64, epsilon = EPSILON);
        assert_abs_diff_eq!(y.values()[n - 1], 0f64, epsilon = EPSILON);
    }

    #[test]
    fn exp() {
        let n: usize = 101;
        let (x0, xf) = (0f64, 2f64);
        let x = linspace(n, x0, xf);
        let ts = TimeSeries1D::exp(&x);
        assert!(ts
            .values()
            .iter()
            .map(|y| y.ln())
            .zip(x)
            .all(|(lny, xi)| relative_eq!(lny, xi)))
    }

    #[test]
    fn flat_line() {
        let n: usize = 100;
        let (x0, xf) = (0f64, 1f64);
        let x = linspace(n, x0, xf);
        let y0: f64 = 10f64;
        let ts = TimeSeries1D::polynomial(&x, vec![y0]);
        assert!(ts.values().iter().all(|yi| yi == &y0));
    }

    #[test]
    fn mx_b() {
        let n: usize = 100;
        let (x0, xf) = (0f64, 1f64);
        let x = linspace(n, x0, xf);
        let y0: f64 = 10f64;
        let m = 0.5f64;
        let yf = y0 + m * xf; // = 10.5
        let ts = TimeSeries1D::polynomial(&x, vec![y0, m]);
        assert!(ts.values().iter().all(|yi| yi >= &y0));
        assert_eq!(ts.values().last(), Some(&yf));
    }

    #[test]
    fn parabola() {
        let n: usize = 100;
        let (x0, xf) = (0f64, 2f64);
        let x = linspace(n, x0, xf);
        let y0 = 10f64;
        let coeffs = vec![y0, 0f64, 1f64];
        let yf = y0 + xf.powf(2.);
        let ts = TimeSeries1D::polynomial(&x, coeffs);
        assert!(ts.values().iter().all(|yi| yi >= &y0));
        assert_eq!(ts.values().last(), Some(&yf));
    }

    #[test]
    fn market() {
        let n: usize = 900;
        let (x0, xf) = (0f64, 6f64 * PI);
        let x = linspace(n, x0, xf);
        let amplitude = 0.5f64;
        let y0 = 10f64;
        let y = TimeSeries1D::sin(&x, amplitude).vertical_shift(y0);
        // let y_sma100 = y.sma(100);
        // let y_sma200 = y.sma(200);
        let y_sma300 = y.sma(300);
        y_sma300
            .values()
            .iter()
            .for_each(|v| assert_abs_diff_eq!(v, &y0, epsilon = EPSILON));
        // assert!(y_sma300.values().iter().all(|v| v == &y0));

        let x2 = x.clone().iter().map(|x| x + PI).collect();
        let z0 = 5f64;
        let z = TimeSeries1D::sin(&x2, amplitude).vertical_shift(z0);
        // let z_sma100 = z.sma(100);
        // let z_sma200 = z.sma(200);
        let z_sma300 = z.sma(300);
        z_sma300
            .values()
            .iter()
            .for_each(|v| assert_abs_diff_eq!(v, &z0, epsilon = EPSILON));
        // assert!(z_sma300.values().iter().all(|v| v == &z0));
    }

    #[test]
    fn mock_data_client_assets() {
        let client = MockDataClient::new();
        // println!("{:?}", client);
        let symbols: HashSet<&Symbol> = client.assets().keys().collect();
        // println!("{:?}", symbols);
        assert_eq!(
            symbols,
            vec![Symbol::from("A"), Symbol::from("B"), Symbol::from("C")]
                .iter()
                .collect()
        )
    }

    #[test]
    fn mock_data_client_query() {
        let client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let asset = Asset::new(Symbol::from("A"));
        let ts = client
            .query(&asset, &MockDataClient::today(), None)
            .unwrap();
        assert_eq!(ts.len(), DATA_SIZE);
        assert_eq!(ts.index().last().unwrap(), &MockDataClient::today());
    }

    #[test]
    fn mock_data_client_query_with_timestamp() {
        let client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let yesterday = MockDataClient::today() - TimeSeries1D::index_unit() * 1 as i32;
        let asset = Asset::new(Symbol::from("A"));
        let ts = client.query(&asset, &yesterday, None).unwrap();

        assert_eq!(ts.len(), DATA_SIZE - 1);
        assert_eq!(ts.index().last().unwrap(), &yesterday);
    }
}
