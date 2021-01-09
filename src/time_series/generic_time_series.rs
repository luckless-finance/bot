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

impl GenTimeSeries<TimeType, KeyType, ValueType> {
    pub(crate) fn intersect(self, other: Self) -> Self {
        for self_t in self.time_series.keys() {
            other.time_series.get(self_t).map(|x| {
                x.keys().all(|k| {
                    self.time_series
                        .get(&self_t)
                        .map_or(false, |m| m.contains_key(k))
                })
            });
            match other.time_series.get(self_t) {
                Some(m) => true,
                _ => false,
            };
        }
        self
    }
    pub(crate) fn intersect2(self, other: Self) -> Self {
        let self_tn = self
            .time_series
            .last_key_value()
            .map_or(&MIN_DATETIME, |item| item.0);
        let other_tn = other
            .time_series
            .last_key_value()
            .map_or(&MIN_DATETIME, |item| item.0);
        let mut self_t = self
            .time_series
            .first_key_value()
            .map_or(&MAX_DATETIME, |item| item.0);
        let mut other_t = other
            .time_series
            .first_key_value()
            .map_or(&MAX_DATETIME, |item| item.0);
        let mut out: BTreeMap<TimeType, HashMap<KeyType, ValueType>> = BTreeMap::new();

        while self_t < self_tn && other_t < other_tn {
            let (t, v) = match self_t.cmp(other_t) {
                Ordering::Less => {
                    // https://stackoverflow.com/a/49600137/5154695
                    // self is missing an element in other
                    // move self_t0 to other_t0
                    other.time_series.range(..self_t).next().unwrap()
                }
                Ordering::Equal => self.time_series.get_key_value(self_t).unwrap(),
                Ordering::Greater => {
                    // move other_t0 to self_t0
                    // other is missing an element
                    other.time_series.range(..self_t).next().unwrap()
                }
            };
        }
        Self {
            name: self.name,
            time_series: out,
        }
    }
}

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

fn merge<T>(lhs: T, rhs: T) -> T
where
    T: Add<Output = T>,
{
    lhs.add(rhs)
}

const MAX_T: i32 = i32::max_value();

trait Limit {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

impl Limit for i32 {
    fn min_value() -> i32 {
        i32::min_value()
    }

    fn max_value() -> i32 {
        i32::max_value()
    }
}

// pub fn gen_join<T, V>(lhs: BTreeMap<T, V>, rhs: BTreeMap<T, V>, merge: fn(V, V) -> V) -> BTreeMap<T, V>
//     where T: Ord + Debug + Limit + Clone,
//           V: Debug
// {
//     let mut lhs_t = lhs.first_key_value().unwrap().0;
//     let lhs_tn = lhs.last_key_value().unwrap().0;
//     let mut rhs_t = rhs.first_key_value().unwrap().0;
//     let rhs_tn = rhs.last_key_value().unwrap().0;
//
//     println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
//     println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);
//
//     let mut i = 0;
//     let mut out: BTreeMap<T, V> = BTreeMap::new();
//     let max_value = T::max_value();
//     while lhs_t <= lhs_tn && rhs_t <= rhs_tn {
//         match lhs_t.cmp(rhs_t) {
//             Ordering::Less => {
//                 println!("\nlhs < rhs , {:?} < {:?}", lhs_t, rhs_t);
//                 // move lhs to rhs
//                 lhs_t = lhs.range(rhs_t..).next().map_or(&max_value, |(t, _)| t);
//                 println!("move so that lhs_t = {:?}", lhs_t);
//             }
//             Ordering::Equal => {
//                 println!("\nlhs = rhs , {:?} = {:?}", lhs_t, rhs_t);
//                 out.insert(rhs_t.clone(), merge(lhs.get(&lhs_t)?, &rhs[&lhs_t]));
//                 println!("out = {:?}", out);
//
//                 let mut lhs_iter = lhs.range(lhs_t..);
//                 lhs_iter.advance_by(1);
//                 lhs_t = lhs_iter.next().map_or(&max_value, |(t, _)| t);
//
//                 let mut rhs_iter = rhs.range(rhs_t..);
//                 rhs_iter.advance_by(1);
//                 rhs_t = rhs_iter.next().map_or(&max_value, |(t, _)| t);
//
//                 println!("move both lhs_t = {:?}; rhs_t = {:?}", lhs_t, rhs_t);
//             }
//             Ordering::Greater => {
//                 println!("\nlhs > rhs , {:?} > {:?}", lhs_t, rhs_t);
//                 // move rhs up to the next largest index after
//                 rhs_t = rhs.range(lhs_t..).next().map_or(&max_value, |(t, _)| t);
//                 println!("move so that rhs_t = {:?}", rhs_t);
//             }
//         }
//         i += 1;
//         if i > 20 {
//             println!("ERROR - infinite loop");
//             break;
//         }
//     }
//     lhs
// }

pub fn join(
    lhs: BTreeMap<i32, i32>,
    rhs: BTreeMap<i32, i32>,
    merge: fn(i32, i32) -> i32,
) -> BTreeMap<i32, i32> {
    let mut lhs_t = lhs.first_key_value().unwrap().0;
    let lhs_tn = lhs.last_key_value().unwrap().0;
    let mut rhs_t = rhs.first_key_value().unwrap().0;
    let rhs_tn = rhs.last_key_value().unwrap().0;

    println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
    println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);

    let mut i = 0;
    let mut out: BTreeMap<i32, i32> = BTreeMap::new();
    // once lhs_t
    while lhs_t <= lhs_tn && rhs_t <= rhs_tn {
        match lhs_t.cmp(rhs_t) {
            Ordering::Less => {
                println!("\nlhs < rhs , {:?} < {:?}", lhs_t, rhs_t);
                // move lhs to rhs
                lhs_t = lhs.range(rhs_t..).next().map_or(&MAX_T, |(t, _)| t);
                println!("move so that lhs_t = {:?}", lhs_t);
            }
            Ordering::Equal => {
                println!("\nlhs = rhs , {:?} = {:?}", lhs_t, rhs_t);
                out.insert(lhs_t.clone(), merge(lhs[&lhs_t], rhs[&lhs_t]));
                println!("out = {:?}", out);

                let mut lhs_iter = lhs.range(lhs_t..);
                lhs_iter.advance_by(1);
                lhs_t = lhs_iter.next().map_or(&MAX_T, |(t, _)| t);

                let mut rhs_iter = rhs.range(rhs_t..);
                rhs_iter.advance_by(1);
                rhs_t = rhs_iter.next().map_or(&MAX_T, |(t, _)| t);

                println!("move both lhs_t = {:?}; rhs_t = {:?}", lhs_t, rhs_t);
            }
            Ordering::Greater => {
                println!("\nlhs > rhs , {:?} > {:?}", lhs_t, rhs_t);
                // move rhs up to the next largest index after
                rhs_t = rhs.range(lhs_t..).next().map_or(&MAX_T, |(t, _)| t);
                println!("move so that rhs_t = {:?}", rhs_t);
            }
        }
        i += 1;
        if i > 20 {
            println!("ERROR - infinite loop");
            break;
        }
    }
    out
}

pub fn join_iter(
    lhs: BTreeMap<i32, i32>,
    rhs: BTreeMap<i32, i32>,
    merge: fn(i32, i32) -> i32,
) -> GenResult<BTreeMap<i32, i32>> {
    let mut lhs_t_iter = lhs.keys();
    let mut lhs_t = match lhs_t_iter.next() {
        Some(t) => t,
        None => &i32::max_value(),
    };
    let lhs_tn = match lhs_t_iter.last() {
        Some(t) => t,
        None => &i32::min_value(),
    };
    let mut rhs_t = rhs.first_key_value().unwrap().0;
    let rhs_tn = rhs.last_key_value().unwrap().0;

    println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
    println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);

    let mut i = 0;
    let mut out: BTreeMap<i32, i32> = BTreeMap::new();
    // once lhs_t
    while lhs_t <= lhs_tn && rhs_t <= rhs_tn {
        match lhs_t.cmp(rhs_t) {
            Ordering::Less => {
                println!("\nlhs < rhs , {:?} < {:?}", lhs_t, rhs_t);
                // move lhs to rhs
                lhs_t = lhs.range(rhs_t..).next().map_or(&MAX_T, |(t, _)| t);
                println!("move so that lhs_t = {:?}", lhs_t);
            }
            Ordering::Equal => {
                println!("\nlhs = rhs , {:?} = {:?}", lhs_t, rhs_t);
                out.insert(lhs_t.clone(), merge(lhs[&lhs_t], rhs[&lhs_t]));
                println!("out = {:?}", out);

                let mut lhs_iter = lhs.range(lhs_t..);
                lhs_iter.advance_by(1);
                lhs_t = lhs_iter.next().map_or(&MAX_T, |(t, _)| t);

                let mut rhs_iter = rhs.range(rhs_t..);
                rhs_iter.advance_by(1);
                rhs_t = rhs_iter.next().map_or(&MAX_T, |(t, _)| t);

                println!("move both lhs_t = {:?}; rhs_t = {:?}", lhs_t, rhs_t);
            }
            Ordering::Greater => {
                println!("\nlhs > rhs , {:?} > {:?}", lhs_t, rhs_t);
                // move rhs up to the next largest index after
                rhs_t = rhs.range(lhs_t..).next().map_or(&MAX_T, |(t, _)| t);
                println!("move so that rhs_t = {:?}", rhs_t);
            }
        }
        i += 1;
        if i > 20 {
            println!("ERROR - infinite loop");
            break;
        }
    }
    Ok(out)
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
    fn btreemap_join_eq_end() -> GenResult<()> {
        let mut lhs = BTreeMap::new();
        lhs.insert(2, 0);
        lhs.insert(3, 1);
        lhs.insert(10, 3);
        lhs.insert(11, 4);
        lhs.insert(13, 4);
        let mut rhs = BTreeMap::new();
        rhs.insert(1, 0);
        rhs.insert(3, 1);
        rhs.insert(4, 2);
        rhs.insert(10, 3);
        rhs.insert(13, 4);
        let mut out = BTreeMap::new();
        out.insert(3, 2);
        out.insert(10, 6);
        out.insert(13, 8);
        assert_eq!(out, join_iter(lhs, rhs, merge)?);
        Ok(())
    }

    #[test]
    fn btreemap_join_eq_start() -> GenResult<()> {
        let mut lhs = BTreeMap::new();
        lhs.insert(1, 0);
        lhs.insert(3, 1);
        lhs.insert(10, 3);
        lhs.insert(11, 4);
        let mut rhs = BTreeMap::new();
        rhs.insert(1, 0);
        rhs.insert(3, 1);
        rhs.insert(4, 2);
        rhs.insert(10, 3);
        rhs.insert(13, 4);
        let mut out = BTreeMap::new();
        out.insert(3, 2);
        out.insert(10, 6);
        // BTreeMap sorts its keys
        out.insert(1, 0);
        assert_eq!(out, join_iter(lhs, rhs, merge)?);
        Ok(())
    }

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
