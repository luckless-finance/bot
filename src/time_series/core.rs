use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};

struct TimeSeries<K, V>
    where K: Ord + Sized,
          V: Sized {
    data: Box<BTreeMap<K, V>>,
}

type TimeSeriesMap = TimeSeries<DateTime<Utc>, HashMap<String, f64>>;
type TimeSeries1D = TimeSeries<DateTime<Utc>, f64>;

impl<K, V> TimeSeries<K, V> {
    pub fn new() -> TimeSeries<K, V>
        where K: Ord + Sized,
              V: Sized
    {
        TimeSeries {
            data: Box::new(BTreeMap::new())
        }
    }
}

impl<K> TimeSeries<K, V>
    where K: Ord + Sized,
          V: f64
{
    pub fn new() -> TimeSeries<K, f64> {
        println!("new 1D ts");
        TimeSeries {
            data: Box::new(BTreeMap::new())
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use chrono::prelude::*;

    use crate::time_series::core::{TimeSeries, TimeSeries1D};

    #[test]
    fn foo() {
        let mut time_series_a = TimeSeries1D::new();
        let mut time_series_b: TimeSeries<DateTime<Utc>, f64> = TimeSeries::new();
        time_series_a.data.insert(Utc::now(), 1.0);
    }
}