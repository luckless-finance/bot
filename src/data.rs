#![allow(dead_code)]

use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::time_series::{DataPointValue, TimeSeries, TimeSeries1D, TimeStamp};

pub struct Asset {}

pub struct MockDataClient {}

impl MockDataClient {
    pub fn new() -> Self {
        MockDataClient {}
    }
    pub fn query(_asset: &Asset, _timestamp: &TimeStamp) -> TimeSeries1D {
        TimeSeries1D::from_values(simulate(100))
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
    use crate::data::{plot, simulate};
    use crate::time_series::TimeSeries1D;

    // #[test]
    fn demo_with_gnuplot() {
        let values = simulate(100);
        let ts = TimeSeries1D::from_values(values);
        plot(ts);
    }
}
