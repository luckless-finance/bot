#![allow(unstable_features)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::ops::Add;

use chrono::{DateTime, Utc};

type TimeSeriesMap = TimeSeries<DateTime<Utc>, HashMap<String, f64>>;
type TimeSeries1D = TimeSeries<DateTime<Utc>, f64>;

#[derive(Debug, Clone, PartialEq)]
struct TimeSeries<K, V>
where
    K: Ord + Sized + Debug + Clone + PartialEq,
    V: Sized + Debug + Clone + PartialEq,
{
    data: BTreeMap<K, V>,
}

impl<K, V> TimeSeries<K, V>
where
    K: Ord + Sized + Debug + Clone + PartialEq,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    pub fn new() -> TimeSeries<K, V> {
        TimeSeries {
            data: BTreeMap::new(),
        }
    }

    pub fn align(&self) -> TimeSeries<K, V> {
        TimeSeries::new()
    }
}

// https://doc.rust-lang.org/std/ops/trait.Add.html
// impl<K, V> Add for TimeSeries<K, V>
//     where
//         K: Ord + Sized + Debug + Clone + PartialEq,
//         V: Sized + Debug + Clone + PartialEq + Add<Output=V>
// {
//     type Output = Self;
//     fn add(self, rhs: Self) -> Self::Output {
//         // self.data.keys()
//         // self.data.append()
//         // let left_index = self.data.first_key_value()?.0;
//         // let right_index = rhs.data.first_key_value()?.0;
//         // let i = match left_index.cmp(&right_index) {
//         //     Ordering::Equal => left_index,
//         //     _ => left_index
//         // };
//         Self::new();
//         Self {
//             data: self.data
//         }
//     }
// }

impl<K> TimeSeries<K, f64> where K: Ord + Sized + Debug + Clone + PartialEq {}

impl<K> Add for TimeSeries<K, f64>
where
    K: Ord + Sized + Debug + Clone + PartialEq,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { data: rhs.data }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::error::Error;
    use std::ops::Add;

    use chrono::prelude::*;

    use crate::time_series::generic_time_series::*;

    #[test]
    fn test_1d() {
        let mut time_series_1d = TimeSeries1D::new();
        let mut time_series: TimeSeries<DateTime<Utc>, f64> = TimeSeries::new();
        time_series_1d.data.insert(Utc::now(), 1.0);
        time_series
            .data
            .insert(time_series_1d.data.keys().next().unwrap().clone(), 1.0);
        assert_eq!(time_series, time_series_1d);
        assert_eq!(time_series.clone() + time_series_1d.clone(), time_series_1d);
    }
}
