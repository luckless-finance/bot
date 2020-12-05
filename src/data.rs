#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use std::ops::Index;

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
            &[Caption("Parabola")],
        );
    fg.show().unwrap();
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::data::{plot, simulate, MockDataClient, DATA_SIZE, TODAY};
    use crate::time_series::TimeSeries1D;

    #[test]
    fn mock_data_client_assets() {
        let client = MockDataClient::new();
        println!("{:?}", client);
        let symbols: HashSet<&String> = client.assets().keys().collect();
        println!("{:?}", symbols);
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
        println!("{:?}", client);
        let asset = client.assets.get("A").unwrap();
        let ts = client.query(asset, &TODAY);
        // println!("{:?}", ts);
        assert_eq!(ts.len(), DATA_SIZE);
    }

    // #[test]
    fn demo_with_gnuplot() {
        let values = simulate(100);
        let ts = TimeSeries1D::from_values(values);
        plot(ts);
    }
}
