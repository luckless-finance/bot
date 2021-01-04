#![allow(unstable_features)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::iter::{FromIterator, FusedIterator};
use std::ops::{Add, Range};

use chrono::{DateTime, Utc};
use itertools::Itertools;

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
    pub fn from_vec(k: Vec<K>, v: Vec<V>) -> TimeSeries<K, V> {
        let x: Range<i32> = 0..0;
        TimeSeries {
            data: BTreeMap::from_iter(k.into_iter().zip(v.into_iter())),
        }
    }
}

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
    use itertools::EitherOrBoth::{Both, Left, Right};
    use itertools::Itertools;

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

    #[test]
    fn merge_by() {
        let bcf = TimeSeries::from_vec(vec!['b', 'c', 'f'], vec![1, 4, 2]).data;
        println!("bcf={:?}", bcf);
        let abc = TimeSeries::from_vec(vec!['a', 'b', 'c'], vec![1, 4, 2]).data;
        println!("abc={:?}", abc);

        let merge_by: Vec<(char, i32)> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .collect();
        println!("merge_by={:?}", merge_by);
        let expected_merge_by = vec![('a', 1), ('b', 4), ('b', 1), ('c', 2), ('c', 4), ('f', 2)];
        assert_eq!(merge_by, expected_merge_by)
    }

    #[test]
    fn merge_by_coalesce() {
        let bcf = TimeSeries::from_vec(vec!['b', 'c', 'f'], vec![1, 4, 2]).data;
        println!("bcf={:?}", bcf);
        let abc = TimeSeries::from_vec(vec!['a', 'b', 'c'], vec![1, 4, 2]).data;
        println!("abc={:?}", abc);

        let merge_by: Vec<(char, i32)> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .coalesce(|a, b| {
                if a.0 == b.0 {
                    Ok((a.0, a.1 + b.1))
                } else {
                    Err((a, b))
                }
            })
            .collect();
        println!("merge_by={:?}", merge_by);
        let expected_merge_by = vec![('a', 1), ('b', 5), ('c', 6), ('f', 2)];
        assert_eq!(merge_by, expected_merge_by)
    }

    #[test]
    fn merge_by_coalesce_to_map() {
        let bcf = TimeSeries::from_vec(vec!['b', 'c', 'f'], vec![1, 4, 2]).data;
        println!("bcf={:?}", bcf);
        let abc = TimeSeries::from_vec(vec!['a', 'b', 'c'], vec![1, 4, 2]).data;
        println!("abc={:?}", abc);

        let merge_by: BTreeMap<char, i32> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .coalesce(|a, b| {
                if a.0 == b.0 {
                    Ok((a.0, a.1 + b.1))
                } else {
                    Err((a, b))
                }
            })
            .collect();
        println!("merge_by={:?}", merge_by);
        assert_eq!(merge_by.first_key_value().unwrap(), (&'a', &1));
        assert_eq!(merge_by.get(&'b').unwrap(), &5);
        assert_eq!(merge_by.last_key_value().unwrap(), (&'f', &2));
    }


    #[test]
    fn merge_by_group_by() {
        let bcf = TimeSeries::from_vec(vec!['b', 'c', 'f'], vec![1, 4, 2]).data;
        println!("bcf={:?}", bcf);
        let abc = TimeSeries::from_vec(vec!['a', 'b', 'c'], vec![1, 4, 2]).data;
        println!("abc={:?}", abc);

        let merge_by: HashMap<char, Vec<(char, i32)>> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .into_group_map_by(|a| a.0);
        println!("merge_by={:?}", merge_by);
        assert_eq!(merge_by[&'a'], vec![('a', 1)]);
        assert_eq!(merge_by[&'b'], vec![('b', 4), ('b', 1)]);
    }
}
