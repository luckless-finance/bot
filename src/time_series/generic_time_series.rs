#![allow(unstable_features)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(unused_mut)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::iter::{FromIterator, FusedIterator};
use std::ops::{Add, Range};

use crate::errors::{GenError, GenResult};
use chrono::{Date, DateTime, Utc, MAX_DATETIME, MIN_DATETIME};
use itertools::traits::HomogeneousTuple;
use itertools::Itertools;
use petgraph::visit::Time;
use std::collections::hash_map::RandomState;
use std::hash::Hash;

const DEFAULT_KEY: &str = "DEFAULT";
const DEFAULT_TIME_SERIES_NAME: &str = "DEFAULT";

#[derive(Debug, Clone)]
struct GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + PartialEq + Hash,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    name: String,
    time_series: BTreeMap<T, HashMap<K, V>>,
}

impl<T, K, V> GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + PartialEq + Hash,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    pub fn new(name: String, time_series: BTreeMap<T, HashMap<K, V>>) -> GenTimeSeries<T, K, V> {
        GenTimeSeries { name, time_series }
    }
    pub fn with_name(self, name: String) -> GenTimeSeries<T, K, V> {
        GenTimeSeries {
            name,
            time_series: self.time_series,
        }
    }
    // TODO implement based on join.rs
    pub fn join(self, other: Self) -> Self {
        self
    }
}

impl<T, K, V> PartialEq for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + PartialEq + Hash,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
    }

    fn ne(&self, _other: &Self) -> bool {
        unimplemented!()
    }
}

impl<T, K, V> FromIterator<(T, Vec<(K, V)>)> for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + PartialEq + Eq + Hash,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    fn from_iter<I: IntoIterator<Item = (T, Vec<(K, V)>)>>(iter: I) -> Self {
        let mut time_series = BTreeMap::new();
        for tuple in iter {
            time_series.insert(tuple.0, tuple.1.into_iter().collect());
        }
        Self {
            name: DEFAULT_TIME_SERIES_NAME.to_string(),
            time_series,
        }
    }
}

type TimeType = DateTime<Utc>;
type KeyType = String;
type ValueType = f64;
type TimeSeries = GenTimeSeries<TimeType, KeyType, ValueType>;

impl Add for GenTimeSeries<TimeType, KeyType, ValueType> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // FIXME validate time axis of self and rhs align
        Self {
            name: format!("add({},{})", self.name, rhs.name),
            // TODO implement add
            time_series: rhs.time_series,
        }
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
    use std::borrow::Borrow;

    #[test]
    fn from_iter() {
        let x = vec![
            (1, vec![("close".to_string(), 12.3)]),
            (3, vec![("close".to_string(), 12.3)]),
        ];
        let ts: GenTimeSeries<i32, String, f64> = x.into_iter().collect();
        assert_eq!(ts.time_series[&1].get("close").unwrap(), &12.3);
        assert_eq!(ts.time_series[&3].get("close").unwrap(), &12.3);
    }

    #[test]
    fn merge_by() {
        let abc = BTreeMap::from_iter(['a', 'b', 'c'].iter().zip([1, 4, 2].iter()));
        println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(['b', 'c', 'f'].iter().zip([1, 4, 2].iter()));
        println!("bcf={:?}", bcf);

        let merge_by: Vec<(&char, &i32)> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .collect();
        println!("merge_by={:?}", merge_by);
        let expected_merge_by = vec![
            (&'a', &1),
            (&'b', &4),
            (&'b', &1),
            (&'c', &2),
            (&'c', &4),
            (&'f', &2),
        ];
        assert_eq!(merge_by, expected_merge_by)
    }

    #[test]
    fn merge_by_coalesce_to_map() {
        let elems: Vec<(char, i32)> = vec!['a', 'b', 'c']
            .into_iter()
            .zip(vec![1, 4, 2].into_iter())
            .collect();
        let abc = BTreeMap::from_iter(elems.into_iter());
        println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(
            vec!['b', 'c', 'f']
                .into_iter()
                .zip(vec![1, 4, 2].into_iter()),
        );
        println!("bcf={:?}", bcf);

        let merge_by: BTreeMap<char, i32> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .coalesce(|a, b| {
                if a.0 == b.0 {
                    // let sum: i32 = *a.1 + *b.1;
                    Ok((a.0, (a.1 + b.1)))
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
        let abc = BTreeMap::from_iter(['a', 'b', 'c'].iter().zip([1, 4, 2].iter()));
        println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(['b', 'c', 'f'].iter().zip([1, 4, 2].iter()));
        println!("bcf={:?}", bcf);

        let merge_by: HashMap<&char, Vec<(&char, &i32)>> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .into_group_map_by(|a| a.0);
        println!("merge_by={:?}", merge_by);
        assert_eq!(merge_by[&'a'], vec![(&'a', &1)]);
        assert_eq!(merge_by[&'b'], vec![(&'b', &4), (&'b', &1)]);
    }
}
