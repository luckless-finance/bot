#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Index;

use gnuplot::{AxesCommon, Figure};
use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};

static DATA_SIZE: usize = 10_000;
pub static TODAY: usize = DATA_SIZE;

pub trait DataClient {
    fn asset(&self, symbol: String) -> Result<&Asset, &str>;
    fn query(&self, asset: &Asset, timestamp: &TimeStamp) -> Result<TimeSeries1D, &str>;
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Asset {
    symbol: String,
}

impl Asset {
    pub fn new(symbol: String) -> Self {
        Asset { symbol }
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug)]
pub struct MockDataClient {
    assets: HashMap<String, Asset>,
}

impl DataClient for MockDataClient {
    fn asset(&self, symbol: String) -> Result<&Asset, &str> {
        Ok(self.assets.index(symbol.as_str()))
    }

    fn query(&self, asset: &Asset, timestamp: &usize) -> Result<TimeSeries1D, &str> {
        assert!(
            self.assets.contains_key(&asset.symbol),
            "query for {} at {} failed",
            asset,
            timestamp
        );
        Ok(TimeSeries1D::from_values(simulate(DATA_SIZE)))
    }
}

impl MockDataClient {
    pub fn new() -> Self {
        MockDataClient {
            assets: vec![
                (String::from("A"), Asset::new(String::from("A"))),
                (String::from("B"), Asset::new(String::from("B"))),
                (String::from("C"), Asset::new(String::from("C"))),
            ]
            .into_iter()
            .collect(),
        }
    }
    // TODO make this private
    pub fn assets(&self) -> &HashMap<String, Asset> {
        &self.assets
    }
    pub fn query(&self, asset: &Asset, timestamp: &TimeStamp) -> TimeSeries1D {
        assert!(
            self.assets.contains_key(&asset.symbol),
            "query for {} at {} failed",
            asset,
            timestamp
        );
        TimeSeries1D::from_values(simulate(DATA_SIZE))
    }
}

fn simulate(limit: usize) -> Vec<DataPointValue> {
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

pub fn plot(time_series: TimeSeries1D) {
    let mut fg = Figure::new();
    fg.axes2d()
        .set_title("Time Series Plot", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("timestamp", &[])
        .set_y_label("values", &[])
        .lines(
            time_series.index(),
            time_series.values(),
            &[Caption("Price")],
        );
    fg.show().unwrap();
}

#[cfg(test)]
mod tests {
    use std::cmp::min;
    use std::collections::HashSet;
    use std::f64::consts::PI;

    use gnuplot::{AxesCommon, Figure};
    use gnuplot::AutoOption::Fix;
    use gnuplot::Coordinate::Graph;
    use gnuplot::PlotOption::Caption;
    use rand_distr::num_traits::{AsPrimitive, Pow};

    use crate::data::{DATA_SIZE, MockDataClient, TODAY};

    #[test]
    fn mock_data_client_assets() {
        let client = MockDataClient::new();
        // println!("{:?}", client);
        let symbols: HashSet<&String> = client.assets().keys().collect();
        // println!("{:?}", symbols);
        assert_eq!(
            symbols,
            vec![String::from("A"), String::from("B"), String::from("C")]
                .iter()
                .collect()
        )
    }

    #[test]
    fn mock_data_client_query() {
        let client = MockDataClient::new();
        // println!("{:?}", client);
        let asset = client.assets.get("A").unwrap();
        let ts = client.query(asset, &TODAY);
        // println!("{:?}", ts);
        assert_eq!(ts.len(), DATA_SIZE);
    }

    pub fn plot(x: Vec<f64>, y: Vec<f64>) {
        let mut fg = Figure::new();
        let y_max: f64 = y.iter().cloned().fold(f64::NAN, f64::max) * 1.1;
        let y_min: f64 = f64::min(y.iter().cloned().fold(f64::NAN, f64::min) * 0.9, 0f64);
        fg.axes2d()
            .set_title("Time Series Plot", &[])
            .set_legend(Graph(0.5), Graph(0.9), &[], &[])
            .set_x_label("timestamp", &[])
            .set_y_label("values", &[])
            .set_y_range(Fix::<f64>(y_min), Fix::<f64>(y_max))
            .lines(
                x,
                y,
                &[Caption("Price")],
            );
        fg.show().unwrap();
    }

    fn sin(x: &[f64], period: f64, amplitude: f64, offset: f64) -> Vec<f64> {
        let x_max: f64 = x.iter().cloned().fold(f64::NAN, f64::max);
        assert_eq!(x_max, 1f64);
        x.iter()
            .map(|x| period * PI * 2.0 * *x as f64 / 1 as f64)
            .map(|x| amplitude * x.sin() + offset)// 1 period
            .collect()
    }

    fn parabola(x: &[f64], amplitude: f64, offset: f64) -> Vec<f64> {
        x.iter()
            .map(|x| *x * 2f64 - 1f64)
            .map(|x| x.pow(2f64))
            .map(|x| amplitude * x + offset)// 1 period
            .collect()
    }

    fn x(n: usize) -> Vec<f64> {
        let (xi, xf) = (0, n);
        (xi..=xf).into_iter().map(|x| x as f64 / xf as f64).collect()
    }

    #[test]
    fn basic_sin_graph() {
        let i: Vec<f64> = x(100);
        let y = sin(&i, 1.as_(), 1.as_(), 0.as_());
        plot(i, y);
    }

    #[test]
    fn sin_graph() {
        let i: Vec<f64> = x(100);
        let y = sin(&i, 2.as_(), 5.as_(), 10.as_());
        plot(i, y);
    }

    #[test]
    fn basic_parabola_graph() {
        let i = x(100);
        let y = parabola(&i, 1.as_(), 0.as_());
        plot(i, y);
    }

    #[test]
    fn parabola_graph() {
        let i = x(100);
        let y = parabola(&i, (-1).as_(), 1.as_());
        plot(i, y);
    }

    #[test]
    fn sin_parabola_graph() {
        let x = x(100);
        let y1 = parabola(&x, (-10).as_(), 1.as_());
        let y2 = sin(&x, 4.as_(), 1.as_(), 20.as_());
        let mut y: Vec<f64> = Vec::with_capacity(100);
        for i in 0..=100 {
            y.insert(i, y1[i] + y2[i]);
        }
        println!("{:?}", x);
        println!("{:?}", y);
        plot(x, y);
    }

    // #[test]
    // fn demo_with_gnuplot() {
    //     let values = simulate(100);
    //     let ts = TimeSeries1D::from_values(values);
    //     plot(ts);
    // }
}
