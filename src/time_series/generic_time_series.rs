#![allow(unstable_features)]
// #![allow(unused_variables)]
// #![allow(unused_must_use)]
// #![allow(unused_mut)]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::{FromIterator, FusedIterator};
use std::ops::{Add, Range};

use crate::errors::{GenError, GenResult, TimeSeriesError};
use chrono::{Date, DateTime, Utc, MAX_DATETIME, MIN_DATETIME};
use core::panicking::panic;
use itertools::traits::HomogeneousTuple;
use itertools::{zip, Itertools};
use petgraph::visit::Time;
use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::hash::Hash;

const DEFAULT_KEY: &str = "DEFAULT";
const DEFAULT_TIME_SERIES_NAME: &str = "DEFAULT";

#[derive(Debug, Clone)]
struct GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    name: String,
    time_series: BTreeMap<T, BTreeMap<K, V>>,
}

pub trait Limits {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

trait Merge {
    fn merge(lhs: &Self, rhs: &Self) -> Self;
}

impl<K, V> Merge for BTreeMap<K, V>
where
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    fn merge(lhs: &Self, rhs: &Self) -> Self {
        let mut out: Self = BTreeMap::from(lhs.clone());
        rhs.iter().for_each(|(a, b)| {
            out.insert(a.clone(), b.clone());
            ()
        });
        out
    }
}

impl<T, K, V> GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord + Limits,
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    pub fn new(name: String, time_series: BTreeMap<T, BTreeMap<K, V>>) -> GenTimeSeries<T, K, V> {
        GenTimeSeries { name, time_series }
    }
    pub fn with_name(self, name: String) -> GenTimeSeries<T, K, V> {
        GenTimeSeries {
            name,
            time_series: self.time_series,
        }
    }

    /// Performs inner join with another instance and returns new joined instance
    /// Returned instance only contains `T` keys that are present in both `self` and `other`.
    /// Runtime: O(min(n,m)) where n,m are the lens of the TimeSeries instances
    pub fn join(self, other: Self) -> Self {
        let lhs = self.time_series;
        let rhs = other.time_series;

        let mut lhs_t = lhs.first_key_value().unwrap().0;
        let lhs_tn = lhs.last_key_value().unwrap().0;
        let mut rhs_t = rhs.first_key_value().unwrap().0;
        let rhs_tn = rhs.last_key_value().unwrap().0;

        // println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
        // println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);

        let max_value = T::max_value();

        let mut i = 0;
        let mut out: BTreeMap<T, BTreeMap<K, V>> = BTreeMap::new();
        while lhs_t <= lhs_tn && rhs_t <= rhs_tn {
            match lhs_t.cmp(rhs_t) {
                Ordering::Less => {
                    // println!("\nlhs < rhs , {:?} < {:?}", lhs_t, rhs_t);
                    // move lhs to rhs
                    lhs_t = lhs.range(rhs_t..).next().map_or(&max_value, |(t, _)| t);
                    // println!("move so that lhs_t = {:?}", lhs_t);
                }
                Ordering::Equal => {
                    // println!("\nlhs = rhs , {:?} = {:?}", lhs_t, rhs_t);
                    // let x = lhs[&lhs_t].borrow();
                    out.insert(lhs_t.clone(), BTreeMap::merge(&rhs[&lhs_t], &rhs[&lhs_t]));
                    // println!("out = {:?}", out);

                    let mut lhs_iter = lhs.range(lhs_t..);
                    lhs_t = match lhs_iter.advance_by(1) {
                        Ok(()) => lhs_iter.next().map_or(&max_value, |(t, _)| t),
                        Err(_) => &max_value,
                    };

                    let mut rhs_iter = rhs.range(rhs_t..);
                    rhs_t = match rhs_iter.advance_by(1) {
                        Ok(()) => rhs_iter.next().map_or(&max_value, |(t, _)| t),
                        Err(_) => &max_value,
                    };

                    // println!("move both lhs_t = {:?}; rhs_t = {:?}", lhs_t, rhs_t);
                }
                Ordering::Greater => {
                    // println!("\nlhs > rhs , {:?} > {:?}", lhs_t, rhs_t);
                    // move rhs up to the next largest index after
                    rhs_t = rhs.range(lhs_t..).next().map_or(&max_value, |(t, _)| t);
                    // println!("move so that rhs_t = {:?}", rhs_t);
                }
            }
            i += 1;
            if i > 20 {
                panic!("ERROR - infinite loop");
            }
        }
        Self {
            name: format!("join({}, {})", self.name, other.name),
            time_series: out,
        }
    }
}

impl<T, K, V> Eq for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
}

impl<T, K, V> PartialEq for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    // FIXME only checks keys
    fn eq(&self, _other: &Self) -> bool {
        self.time_series.keys().eq(_other.time_series.keys())
    }
    // FIXME only checks keys
    fn ne(&self, _other: &Self) -> bool {
        self.time_series.keys().ne(_other.time_series.keys())
    }
}

impl<T, K, V> FromIterator<(T, Vec<(K, V)>)> for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + Eq + Ord,
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

impl<T, K, V> Add for GenTimeSeries<T, K, V>
where
    T: Sized + Debug + Clone + PartialEq + Ord,
    K: Sized + Debug + Clone + Eq + Ord,
    V: Sized + Debug + Clone + PartialEq + Add<Output = V>,
{
    type Output = GenResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        // FIXME validate time axis of self and rhs align
        match self.time_series.keys().eq(rhs.time_series.keys()) {
            false => Err(TimeSeriesError::new(format!(
                "Error: unable to add: {} + {}; inconsistent time indices",
                self.name, rhs.name
            ))),
            true => {
                // self.time_series.iter().zip(rhs.time_series.iter())
                //     .map(|(l,r)|l.1.iter().zip())
                Ok(Self {
                    name: format!("add({},{})", self.name, rhs.name),
                    // TODO implement add
                    time_series: rhs.time_series,
                })
            }
        }
    }
}

type TimeType = DateTime<Utc>;
type KeyType = String;
type ValueType = f64;
type TimeSeries = GenTimeSeries<TimeType, KeyType, ValueType>;
//
// impl Add for GenTimeSeries<TimeType, KeyType, ValueType> {
//     type Output = Self;
//
//     fn add(self, rhs: Self) -> Self::Output {
//         // FIXME validate time axis of self and rhs align
//         Self {
//             name: format!("add({},{})", self.name, rhs.name),
//             // TODO implement add
//             time_series: rhs.time_series,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::error::Error;
    use std::ops::Add;

    use chrono::prelude::*;
    use itertools::EitherOrBoth::{Both, Left, Right};
    use itertools::Itertools;

    use crate::time_series::generic_time_series::*;
    use std::borrow::Borrow;

    impl Limits for i32 {
        fn min_value() -> Self {
            i32::min_value()
        }

        fn max_value() -> Self {
            i32::max_value()
        }
    }

    #[test]
    fn join() {
        let lhs: GenTimeSeries<i32, &str, f64> = vec![
            (2, vec![(DEFAULT_KEY, 12.3)]),
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            (11, vec![(DEFAULT_KEY, 12.3)]),
            (13, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        let rhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_KEY, 12.3)]),
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (4, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            (13, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        let out: GenTimeSeries<i32, &str, f64> = vec![
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            (13, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        assert_eq!(out, lhs.join(rhs));
    }

    #[test]
    fn btreemap_join_eq_start() -> GenResult<()> {
        let lhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_KEY, 12.3)]),
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            (11, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        let rhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_KEY, 12.3)]),
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (4, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            (13, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        let out: GenTimeSeries<i32, &str, f64> = vec![
            (3, vec![(DEFAULT_KEY, 12.3)]),
            (10, vec![(DEFAULT_KEY, 12.3)]),
            // out of order inserts OK, BTreeMap sorts its keys
            (1, vec![(DEFAULT_KEY, 12.3)]),
        ]
        .into_iter()
        .collect();
        assert_eq!(out, lhs.join(rhs));
        Ok(())
    }

    #[test]
    fn from_iter() {
        let x = vec![
            (1, vec![(DEFAULT_KEY, 12.3)]),
            (3, vec![(DEFAULT_KEY, 12.3)]),
        ];
        let ts: GenTimeSeries<i32, &str, f64> = x.into_iter().collect();
        assert_eq!(ts.time_series[&1].get(DEFAULT_KEY).unwrap(), &12.3);
        assert_eq!(ts.time_series[&3].get(DEFAULT_KEY).unwrap(), &12.3);
    }

    #[test]
    fn merge_by() {
        let abc = BTreeMap::from_iter(['a', 'b', 'c'].iter().zip([1, 4, 2].iter()));
        // println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(['b', 'c', 'f'].iter().zip([1, 4, 2].iter()));
        // println!("bcf={:?}", bcf);

        let merge_by: Vec<(&char, &i32)> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .collect();
        // println!("merge_by={:?}", merge_by);
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
        // println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(
            vec!['b', 'c', 'f']
                .into_iter()
                .zip(vec![1, 4, 2].into_iter()),
        );
        // println!("bcf={:?}", bcf);

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
        // println!("merge_by={:?}", merge_by);
        assert_eq!(merge_by.first_key_value().unwrap(), (&'a', &1));
        assert_eq!(merge_by.get(&'b').unwrap(), &5);
        assert_eq!(merge_by.last_key_value().unwrap(), (&'f', &2));
    }

    #[test]
    fn merge_by_group_by() {
        let abc = BTreeMap::from_iter(['a', 'b', 'c'].iter().zip([1, 4, 2].iter()));
        // println!("abc={:?}", abc);
        let bcf = BTreeMap::from_iter(['b', 'c', 'f'].iter().zip([1, 4, 2].iter()));
        // println!("bcf={:?}", bcf);

        let merge_by: HashMap<&char, Vec<(&char, &i32)>> = abc
            .into_iter()
            .merge_by(bcf.into_iter(), |a, b| a.0 <= b.0)
            .into_group_map_by(|a| a.0);
        // println!("merge_by={:?}", merge_by);
        assert_eq!(merge_by[&'a'], vec![(&'a', &1)]);
        assert_eq!(merge_by[&'b'], vec![(&'b', &4), (&'b', &1)]);
    }
}
