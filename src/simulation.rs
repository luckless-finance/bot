use crate::data::{Asset, AssetNotFoundError, DataClient, Symbol};
use crate::strategy::{GenResult, QueryCalculationDto};
use crate::time_series::TimeSeries1D;
use std::collections::HashMap;
use std::f64::consts::PI;

pub static DATA_SIZE: usize = 10_000;
pub static TODAY: usize = DATA_SIZE;

#[derive(Debug)]
pub struct MockDataClient {
    assets: HashMap<Symbol, Asset>,
}

fn simulate_time_series(n: usize) -> TimeSeries1D {
    let (x0, xf) = (0f64, 6f64 * PI);
    let x = linspace(n, x0, xf);
    let amplitude = 0.5f64;
    let y0 = 10f64;
    TimeSeries1D::sin(&x, amplitude).vertical_shift(y0)
}

impl DataClient for MockDataClient {
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset> {
        match self.assets.get(symbol) {
            Some(asset) => Ok(asset),
            None => Err(Box::new(AssetNotFoundError::new(symbol.clone()))),
        }
    }

    #[allow(unused_variables)]
    fn query(
        &self,
        asset: &Asset,
        timestamp: &usize,
        query_dto: Option<QueryCalculationDto>,
    ) -> GenResult<TimeSeries1D> {
        assert!(
            self.assets.contains_key(&asset.symbol().to_string()),
            "query for {} at {} failed",
            asset,
            timestamp
        );
        Ok(simulate_time_series(DATA_SIZE))
    }
}

impl MockDataClient {
    pub fn new() -> Self {
        MockDataClient {
            assets: vec![
                (Symbol::from("A"), Asset::new(Symbol::from("A"))),
                (Symbol::from("B"), Asset::new(Symbol::from("B"))),
                (Symbol::from("C"), Asset::new(Symbol::from("C"))),
            ]
            .into_iter()
            .collect(),
        }
    }
    // TODO make this private
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
        TimeSeries1D::new(self.index().clone(), y)
    }

    /// Shift `TimeSeries` vertically by `delta`.
    fn vertical_shift(&self, delta: f64) -> Self {
        TimeSeries1D::new(
            self.index().clone(),
            self.values().iter().map(|v| v + delta).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{plot_ts, plots};
    use crate::simulation::*;
    use crate::strategy::GenResult;
    use crate::time_series::TimeSeries1D;
    use rand_distr::num_traits::{AsPrimitive, Pow};
    use std::f64::consts::PI;

    const EPSILON: f64 = 1E-10;

    #[test]
    fn linspace_0_10() -> GenResult<()> {
        let ten = linspace(10, 0.as_(), 10.as_());
        assert_eq!(ten.len(), 10);
        assert_eq!(ten[0], 0f64);
        assert_eq!(ten[9], 10f64);
        Ok(())
    }

    #[test]
    fn linspace_0_2pi() -> GenResult<()> {
        let x = linspace(100, 0.as_(), PI * 2f64);
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
        println!("{:?}", y);
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
        let yf = y0 + xf.pow(2);
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
        let y_sma100 = y.sma(100);
        let y_sma200 = y.sma(200);
        let y_sma300 = y.sma(300);

        let x2 = x.clone().iter().map(|x| x + PI).collect();
        let z0 = 5f64;
        let z = TimeSeries1D::sin(&x2, amplitude).vertical_shift(z0);
        let z_sma100 = z.sma(100);
        let z_sma200 = z.sma(200);
        let z_sma300 = z.sma(300);

        plot_ts(vec![&y, &y_sma100, &y_sma200, &z]);
    }
}