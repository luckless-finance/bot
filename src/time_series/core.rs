use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};

type TimeSeriesMap = TimeSeries<DateTime<Utc>, HashMap<String, f64>>;
type TimeSeries1D = TimeSeries<DateTime<Utc>, f64>;

#[derive(Debug, Clone, PartialEq)]
struct TimeSeries<K, V>
where
    K: Ord + Sized,
    V: Sized,
{
    data: BTreeMap<K, V>,
}

impl<K, V> TimeSeries<K, V>
where
    K: Ord + Sized,
    V: Sized,
{
    pub fn new() -> TimeSeries<K, V> {
        TimeSeries {
            data: BTreeMap::new(),
        }
    }
}

impl<K> TimeSeries<K, f64> where K: Ord + Sized {}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::error::Error;
    use std::ops::Add;

    use chrono::prelude::*;

    use crate::time_series::core::{TimeSeries, TimeSeries1D};

    #[test]
    fn bar() {
        let x = 5;
        let y = x.add(2);
    }

    #[test]
    fn test_1d() {
        let mut time_series_1d = TimeSeries1D::new();
        let mut time_series: TimeSeries<DateTime<Utc>, f64> = TimeSeries::new();
        time_series_1d.data.insert(Utc::now(), 1.0);
        time_series
            .data
            .insert(time_series_1d.data.keys().next().unwrap().clone(), 1.0);
        assert_eq!(time_series, time_series_1d);
    }
}
