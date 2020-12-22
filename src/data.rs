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

pub(crate) static DATA_SIZE: usize = 10_000;
pub(crate) static TODAY: usize = DATA_SIZE;
pub(crate) type Symbol = String;

pub(crate) trait DataClient {
    fn asset(&self, symbol: Symbol) -> Result<&Asset, &str>;
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query: Option<QueryCalculationDto>,
    ) -> GenResult<TimeSeries1D>;
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct Asset {
    symbol: Symbol,
}

impl Asset {
    pub fn new(symbol: Symbol) -> Self {
        Asset { symbol }
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug)]
pub(crate) struct MockDataClient {
    assets: HashMap<Symbol, Asset>,
}

impl DataClient for MockDataClient {
    fn asset(&self, symbol: Symbol) -> Result<&Asset, &str> {
        Ok(self.assets.index(symbol.as_str()))
    }

    #[allow(unused_variables)]
    fn query(
        &self,
        asset: &Asset,
        timestamp: &usize,
        query_dto: Option<QueryCalculationDto>,
    ) -> GenResult<TimeSeries1D> {
        assert!(
            self.assets.contains_key(&asset.symbol),
            "query for {} at {} failed",
            asset,
            timestamp
        );
        Ok(TimeSeries1D::from_values(simulate_random(DATA_SIZE)))
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

fn simulate_random(limit: usize) -> Vec<DataPointValue> {
    let mut x: DataPointValue = 100.0;
    let mut values: Vec<DataPointValue> = Vec::new();
    let mut ran = thread_rng();
    let log_normal = Normal::new(0.0, 1.0).unwrap();
    for _i in 0..limit {
        x = log_normal.sample(&mut ran) + x;
        values.push(x);
    }
    values
}

fn simulate_trig(limit: usize) -> Vec<DataPointValue> {
    let x = x(limit);
    let y1 = sin(&x, (0.5).as_(), 50.as_(), 0.as_());
    let y2 = sin(&x, 4.as_(), 10.as_(), 0.as_());
    let offset = polynomial(&x, &[100f64]);
    sum(&[y1.as_slice(), y2.as_slice(), offset.as_slice()])
}

pub(crate) fn plots(x: Vec<f64>, ys: Vec<Vec<f64>>) {
    let mut fg = Figure::new();
    let y_max: f64 = ys
        .as_slice()
        .iter()
        .flatten()
        .fold(f64::NAN, |a, b| f64::max(a, *b).clone())
        * 1.1;
    let mut y_min = ys
        .as_slice()
        .iter()
        .flatten()
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
        axis.lines(x.clone(), ys.get(i).unwrap(), &[Caption(&format!("{}", i))]);
    }
    fg.show().unwrap();
}

pub(crate) fn plot(x: Vec<f64>, y: Vec<f64>) {
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
        .lines(x.clone(), y, &[Caption("Price")]);
    fg.show().unwrap();
}

fn sin(x: &[f64], period: f64, amplitude: f64, offset: f64) -> Vec<f64> {
    let x_max: f64 = x.iter().cloned().fold(f64::NAN, f64::max);
    assert_eq!(x_max, 1f64);
    x.iter()
        .map(|x| period * PI * 2.0 * *x as f64 / 1 as f64)
        .map(|x| amplitude * x.sin() + offset) // 1 period
        .collect()
}

fn parabola(x: &[f64], amplitude: f64, offset: f64) -> Vec<f64> {
    x.iter()
        .map(|x| *x * 2f64 - 1f64)
        .map(|x| x.pow(2f64))
        .map(|x| amplitude * x + offset) // 1 period
        .collect()
}

fn add(a: &[f64], b: &[f64]) -> Vec<f64> {
    assert_eq!(a.len(), b.len());
    let mut y: Vec<f64> = Vec::with_capacity(a.len());
    for i in 0..=a.len() {
        y.insert(i, a[i] + b[i]);
    }
    y
}

fn sum(v: &[&[f64]]) -> Vec<f64> {
    let mut y: Vec<f64> = Vec::with_capacity(v[0].len());
    for i in 0..v[0].len() {
        let mut temp = 0f64;
        for j in 0..v.len() {
            temp += v[j][i];
        }
        y.insert(i, temp);
    }
    y
}

// TODO proper linspace builder
fn x(n: usize) -> Vec<f64> {
    let (xi, xf) = (0, n);
    (xi..xf)
        .into_iter()
        .map(|x| x as f64 / (xf - 1) as f64)
        .collect()
}

fn _polynomial(xi: &f64, coefficients: &[f64]) -> f64 {
    let mut sum = 0f64;
    for i in 0..coefficients.len() {
        // println!("{} + {}**{}", sum, xi, i);
        sum = sum + coefficients[i] * xi.pow(i as f64);
    }
    // println!("sum={:?}", &sum);
    sum
}

fn polynomial(xi: &[f64], coefficients: &[f64]) -> Vec<f64> {
    xi.iter().map(|x| _polynomial(x, coefficients)).collect()
}

#[cfg(test)]
mod tests {
    // silence approx lib warnings
    #![allow(unused_must_use)]

    use std::collections::HashSet;

    use rand_distr::num_traits::AsPrimitive;

    use crate::data::*;

    const EPSILON: f64 = 1E-10;

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
        // println!("{:?}", client);
        let asset = Asset::new(Symbol::from("A"));
        let ts = client.query(&asset, &TODAY, None).unwrap();
        // println!("{:?}", ts);
        assert_eq!(ts.len(), DATA_SIZE);
    }

    #[test]
    fn lin_space_test() {
        assert!(x(1)[0].is_nan());
        assert_eq!(x(2)[0], 0f64);
        assert_eq!(x(3)[0], 0f64);
        assert_eq!(x(2).len(), 2);
        assert_eq!(x(3).len(), 3);
        assert_eq!(x(100).len(), 100);
        assert_eq!(x(2)[1], 1f64);
        assert_eq!(x(3)[2], 1f64);
        assert_eq!(x(100)[99], 1f64);
        assert_eq!(x(6), vec![0f64, 0.2f64, 0.4f64, 0.6f64, 0.8f64, 1.0f64])
    }

    #[test]
    fn vector_sum() {
        let a = x(100);
        let b = x(100);
        let c = x(100);
        let d = &[a.as_slice(), b.as_slice(), c.as_slice()];
        let s = sum(d);
        for i in 0..a.len() {
            assert_eq!(s[i], a[i] * 3f64)
        }
    }

    #[test]
    fn test_sin() {
        let x = x(101);
        let y = sin(&x, 1.as_(), 1.as_(), 0.as_());
        assert_abs_diff_eq!(y[0], 0f64, epsilon = EPSILON);
        assert_abs_diff_eq!(y[25], 1f64, epsilon = EPSILON);
        assert_abs_diff_eq!(y[50], 0f64, epsilon = EPSILON);
        assert_abs_diff_eq!(y[75], -1f64, epsilon = EPSILON);
        assert_abs_diff_eq!(y[100], 0f64, epsilon = EPSILON);
    }

    #[test]
    fn scalar_polynomial() {
        assert_eq!(_polynomial(&0f64, &[0f64]), 0f64);
        assert_eq!(_polynomial(&0f64, &[1f64]), 1f64);
        assert_eq!(_polynomial(&0f64, &[1f64, 1f64]), 1f64);
        assert_eq!(_polynomial(&5f64, &[0f64, 1f64]), 5f64);
    }

    #[test]
    fn vector_polynomial() {
        let xi = x(100);
        let y = polynomial(&xi, &[0f64, 1f64]);
        assert_eq!(xi, y);
    }

    // #[test]
    fn basic_sin_graph() {
        let i: Vec<f64> = x(100);
        let y = sin(&i, 1.as_(), 1.as_(), 0.as_());
        plot(i, y);
    }

    // #[test]
    fn sin_graph() {
        let i: Vec<f64> = x(100);
        let y = sin(&i, 2.as_(), 5.as_(), 10.as_());
        plot(i, y);
    }

    // #[test]
    fn multi_sin_graph() {
        let x = x(100);
        let offset = 100f64;
        let y1 = sin(&x, (0.5).as_(), 50.as_(), 0.as_());
        let y2 = sin(&x, 4.as_(), 10.as_(), 0.as_());
        let y11 = y1.clone();
        let y22 = y2.clone();
        let mut y: Vec<f64> = Vec::with_capacity(100);
        for i in 0..100 {
            y.insert(i, y1[i] + y2[i] + offset);
        }
        println!("{:?}", x);
        println!("{:?}", y);
        plots(x, vec![y, y11, y22]);
    }

    #[test]
    fn flat() {
        let a = x(100);
        let b = x(100);
        let l = a.len() + b.len();
        let c = vec![a, b];
        let f: Vec<&f64> = c.iter().flatten().collect();
        assert_eq!(f.len(), l);

        let a = x(100);
        let b = x(100);
        let l = a.len() + b.len();
        let c = vec![a, b];
        let f: Vec<&f64> = c.iter().flat_map(|x| x.iter()).collect();
        assert_eq!(f.len(), l);
    }
}
