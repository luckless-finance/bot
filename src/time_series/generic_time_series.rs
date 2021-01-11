#![allow(unstable_features)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Debug;
use std::iter::{FromIterator, FusedIterator};
use std::ops::{Add, Div, Mul, Range, Sub};

use crate::errors::{GenError, GenResult, TimeSeriesError};
use chrono::{Date, DateTime, Utc, MAX_DATETIME, MIN_DATETIME};
use core::panicking::panic;
use itertools::traits::HomogeneousTuple;
use itertools::{zip, Itertools};
use num_traits::real::Real;
use num_traits::Num;
use std::array::IntoIter;
use std::borrow::Borrow;
use std::collections::btree_map::{IntoKeys, Iter, Keys};
use std::collections::hash_map::RandomState;
use std::hash::Hash;

/// Default for `K` in `GenTimeSeries<T, K, V>`
const DEFAULT_INNER_KEY: &str = "DEFAULT";
/// Default for `name` field in `GenTimeSeries`
const DEFAULT_TIME_SERIES_NAME: &str = "DEFAULT";

/// Outer key of [`GenTimeSeries`](struct.GenTimeSeries.html)
pub trait Time = Sized + Debug + Clone + Ord + PartialEq + Limits;

/// Inner key of [`GenTimeSeries`](struct.GenTimeSeries.html); inner series/dimension name
pub trait InnerKey = Sized + Debug + Clone + Ord + Eq;

/// Inner value of [`GenTimeSeries`](struct.GenTimeSeries.html)
pub trait Value<V> = Sized + Debug + Clone + Num;

/// n-dimensional time series structured as nested `BTreeMap`:
/// - outer index is time `T` (eg "2021-01-10")
/// - inner index is key `K` (eg "close")
/// - inner value `V` (eg 23.04)
/// For example, price data of a single stock could be structured as
/// ```json
/// {
///     "2021-01-10": {"close": 3, "high": 4, "low": 1, "open": 2},
///     "2021-01-11": {"close": 4, "high": 5, "low": 2, "open": 3},
///     "2021-01-12": {"close": 5, "high": 6, "low": 3, "open": 4}
/// }
/// ```
/// TimeSeries are used by `Bot` to cache intermediate calculations for a single stock.  `strategy.yaml` could result
/// ```json
/// {
///     "2021-01-10": {"close": 3, "sma50": 4, "sma200": 1, "sma_diff": 2, "score": 2},
///     "2021-01-11": {"close": 4, "sma50": 5, "sma200": 2, "sma_diff": 3, "score": 3},
///     "2021-01-12": {"close": 5, "sma50": 6, "sma200": 3, "sma_diff": 4, "score": 4},
///     ...
/// }
/// ```
/// # TimeSeries are Dense
///
/// Note that GenTimeSeries is dense: every `t` in `T` has the same set of keys `K`
/// so this would be an error
/// ```json
/// {
///     "2021-01-10": {"close": 3, "high": 4, "low": 1, "open": 2},
///     "2021-01-11": {"close": 4, "high": 5, "low": 2},
///     "2021-01-12": {"close": 5, "high": 6, "low": 3, "open": 4}
/// }
/// ```
/// because "open" is missing for "2021-01-11".
#[derive(Debug, Clone)]
pub struct GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    name: String,
    time_series: BTreeMap<T, BTreeMap<K, V>>,
}

/// Specify `min_value` and `max_value` for `T`; used in [`join`](struct.GenTimeSeries.html#method.join)
pub trait Limits {
    /// minimum value allowed for this type
    fn min_value() -> Self;
    /// maximum value allowed for this type
    fn max_value() -> Self;
}

trait Merge {
    fn merge(lhs: &Self, rhs: &Self) -> Self;
}

impl<K, V> Merge for BTreeMap<K, V>
where
    K: InnerKey,
    V: Value<V>,
{
    // TODO return GenResult, Err if keys are not disjoint
    fn merge(lhs: &Self, rhs: &Self) -> Self {
        // println!("merging lhs={:?} with rhs={:?}", lhs, rhs);
        // TODO init out with longer of lhs and rhs
        //  insert shorter of lhs and rhs with loop
        let mut out: Self = BTreeMap::new();
        lhs.iter().for_each(|(k, v)| {
            out.insert(k.clone(), v.clone());
        });
        rhs.iter().for_each(|(k, v)| {
            out.insert(k.clone(), v.clone());
        });
        out
    }
}

impl<T, K, V> GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    /// Create new instance from given time series data.
    pub fn new(name: String, time_series: BTreeMap<T, BTreeMap<K, V>>) -> GenTimeSeries<T, K, V> {
        GenTimeSeries { name, time_series }
    }
    /// Name of the TimeSeries, for example "MSFT".
    pub fn name(&self) -> &str {
        &self.name
    }
    /// For testing purposes
    pub(crate) fn empty() -> GenTimeSeries<T, K, V> {
        GenTimeSeries::new(DEFAULT_TIME_SERIES_NAME.to_string(), BTreeMap::new())
    }
    /// Move to new instance with different name
    pub fn with_name(self, name: &str) -> GenTimeSeries<T, K, V> {
        GenTimeSeries {
            name: name.to_string(),
            time_series: self.time_series,
        }
    }
    /// Number of dates `T` in outer index
    pub fn len(&self) -> usize {
        self.time_series.len()
    }
    /// Set of `K` inner keys at each time in the outer index
    pub fn keys(&self) -> BTreeSet<&K> {
        match self.time_series.first_key_value() {
            Some((_, m)) => m.keys().clone().into_iter().collect(),
            None => BTreeSet::new(),
        }
    }
    /// Performs inner join with another instance and returns new (dense) joined instance
    /// Returned instance only contains `T` keys that are present in both `self` and `other`.
    /// Runtime: O(min(n,m)) where n,m are the lens of the TimeSeries instances
    pub fn join(self, other: Self) -> GenResult<Self> {
        let lhs = self.time_series;
        let rhs = other.time_series;
        let min_value = T::min_value();
        let max_value = T::max_value();
        let mut lhs_t = match lhs.first_key_value() {
            Some((k, _)) => k,
            None => &max_value,
        };
        let lhs_tn = match lhs.last_key_value() {
            Some((k, _)) => k,
            None => &min_value,
        };
        let mut rhs_t = match rhs.first_key_value() {
            Some((k, _)) => k,
            None => &max_value,
        };
        let rhs_tn = match rhs.last_key_value() {
            Some((k, _)) => k,
            None => &min_value,
        };

        // println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
        // println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);

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
                    out.insert(lhs_t.clone(), BTreeMap::merge(&rhs[&lhs_t], &lhs[&lhs_t]));
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
        // TODO should this return Err instead of an empty GenTimeSeries
        Ok(Self {
            name: format!("join({},{})", self.name, other.name),
            time_series: out,
        })
    }
    /// Filters inner `BTreeMap` instances to a single `K`.
    pub fn select(self, selector: K) -> GenResult<GenTimeSeries<T, K, V>> {
        let out: GenTimeSeries<T, K, V> = self
            .time_series
            .iter()
            .flat_map(|(t, m)| match m.get(&selector) {
                Some(v) => Ok((
                    t.clone(),
                    BTreeMap::<_, _>::from_iter(IntoIter::new([(selector.clone(), v.clone())])),
                )),
                None => Err(TimeSeriesError::new("".to_string())),
            })
            .collect();
        Ok(out)
    }
    /// Used to implement traits `Add`, `Sub`, `Mul`, and `Div`.
    pub fn apply(self, other: Self, fun: fn(V, V) -> GenResult<V>) -> GenResult<Self> {
        match self.keys() == other.keys() {
            true => Ok(()),
            false => Err(TimeSeriesError::new(format!(
                "Error: unable to add: {} + {}; inconsistent inner indices",
                self.name, other.name
            ))),
        }?;

        match self.time_series.keys().eq(other.time_series.keys()) {
            false => Err(TimeSeriesError::new(format!(
                "Error: unable to add: {} + {}; inconsistent time indices",
                self.name, other.name
            ))),
            true => {
                // zip on time
                let time_series = self
                    .time_series
                    .iter()
                    .zip(other.time_series.iter())
                    .map(|((t, lhs), (_, rhs))| {
                        (t.clone(), {
                            // TODO add informative message
                            // assert!(lhs.keys().eq(rhs.keys()));
                            // zip on keys
                            lhs.iter()
                                .zip(rhs.iter())
                                .map(|((k, a), (_, b))| {
                                    (k.clone(), fun(a.clone(), b.clone()).expect("FIX ME"))
                                })
                                .collect()
                        })
                    })
                    .collect();
                Ok(Self {
                    name: format!("add({},{})", self.name, other.name),
                    time_series,
                })
            }
        }
    }
}

impl<T, K, V: 'static> Eq for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
}

impl<T, K, V: 'static> PartialEq for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name) && self.time_series.eq(&other.time_series)
    }
    fn ne(&self, other: &Self) -> bool {
        self.name.eq(&other.name) && self.time_series.eq(&other.time_series)
    }
}
// TODO validate inner keys are consistent over time
impl<T, K, V> FromIterator<(T, Vec<(K, V)>)> for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
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

impl<T, K, V> FromIterator<(T, BTreeMap<K, V>)> for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    fn from_iter<I: IntoIterator<Item = (T, BTreeMap<K, V>)>>(iter: I) -> Self {
        let mut time_series = BTreeMap::new();
        for tuple in iter {
            time_series.insert(tuple.0, tuple.1);
        }
        Self {
            name: DEFAULT_TIME_SERIES_NAME.to_string(),
            time_series,
        }
    }
}

impl<T, K, V> Add for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    type Output = GenResult<Self>;

    fn add(self, other: Self) -> Self::Output
    where
        V: Add<Output = V>,
    {
        fn fun<Z>(lhs: Z, rhs: Z) -> GenResult<Z>
        where
            Z: Add<Output = Z>,
        {
            Ok(lhs.add(rhs))
        }
        self.apply(other, fun)
    }
}

impl<T, K, V> Sub for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    type Output = GenResult<Self>;

    fn sub(self, other: Self) -> Self::Output
    where
        V: Sub<Output = V>,
    {
        fn fun<Z>(lhs: Z, rhs: Z) -> GenResult<Z>
        where
            Z: Sub<Output = Z>,
        {
            Ok(lhs.sub(rhs))
        }
        self.apply(other, fun)
    }
}

impl<T, K, V> Mul for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    type Output = GenResult<Self>;

    fn mul(self, other: Self) -> Self::Output
    where
        V: Mul<Output = V>,
    {
        fn fun<Z>(lhs: Z, rhs: Z) -> GenResult<Z>
        where
            Z: Mul<Output = Z>,
        {
            Ok(lhs.mul(rhs))
        }
        self.apply(other, fun)
    }
}

impl<T, K, V> Div for GenTimeSeries<T, K, V>
where
    T: Time,
    K: InnerKey,
    V: Value<V>,
{
    type Output = GenResult<Self>;

    fn div(self, other: Self) -> Self::Output
    where
        V: Div<Output = V>,
    {
        fn fun<Z>(lhs: Z, rhs: Z) -> GenResult<Z>
        where
            Z: Div<Output = Z>,
        {
            Ok(lhs.div(rhs))
        }
        self.apply(other, fun)
    }
}

type TimeType = DateTime<Utc>;
type KeyType = String;
type ValueType = f64;
type TimeSeries = GenTimeSeries<TimeType, KeyType, ValueType>;

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
    fn from_iter() {
        let x = vec![
            (1, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (3, vec![(DEFAULT_INNER_KEY, 12.3)]),
        ];
        let ts: GenTimeSeries<i32, &str, f64> = x.into_iter().collect();
        assert_eq!(ts.time_series[&1].get(DEFAULT_INNER_KEY).unwrap(), &12.3);
        assert_eq!(ts.time_series[&3].get(DEFAULT_INNER_KEY).unwrap(), &12.3);
    }

    mod join {
        use crate::errors::GenResult;
        use crate::time_series::generic_time_series::{GenTimeSeries, DEFAULT_INNER_KEY};

        #[test]
        fn join_eq_end() -> GenResult<()> {
            let lhs_name = "LHS";
            let lhs: GenTimeSeries<i32, &str, f64> = vec![
                (2, vec![(lhs_name, 10.3)]),
                (3, vec![(lhs_name, 10.3)]),
                (10, vec![(lhs_name, 10.3)]),
                (11, vec![(lhs_name, 10.3)]),
                (13, vec![(lhs_name, 10.3)]),
            ]
            .into_iter()
            .collect();
            let rhs_name = "RHS";
            let rhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(rhs_name, 5.3)]),
                (3, vec![(rhs_name, 5.3)]),
                (4, vec![(rhs_name, 5.3)]),
                (10, vec![(rhs_name, 5.3)]),
                (13, vec![(rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let out: GenTimeSeries<i32, &str, f64> = vec![
                (3, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (10, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (13, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }

        #[test]
        fn join_eq_start() -> GenResult<()> {
            let lhs_name = "LHS";
            let lhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(lhs_name, 10.3)]),
                (3, vec![(lhs_name, 10.3)]),
                (10, vec![(lhs_name, 10.3)]),
                (11, vec![(lhs_name, 10.3)]),
                (12, vec![(lhs_name, 10.3)]),
            ]
            .into_iter()
            .collect();
            let rhs_name = "RHS";
            let rhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(rhs_name, 5.3)]),
                (3, vec![(rhs_name, 5.3)]),
                (4, vec![(rhs_name, 5.3)]),
                (10, vec![(rhs_name, 5.3)]),
                (13, vec![(rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let out: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (3, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (10, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }

        #[test]
        fn join_eq_ends() -> GenResult<()> {
            let lhs_name = "LHS";
            let lhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(lhs_name, 10.3)]),
                (3, vec![(lhs_name, 10.3)]),
                (10, vec![(lhs_name, 10.3)]),
                (11, vec![(lhs_name, 10.3)]),
                (13, vec![(lhs_name, 10.3)]),
            ]
            .into_iter()
            .collect();
            let rhs_name = "RHS";
            let rhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(rhs_name, 5.3)]),
                (3, vec![(rhs_name, 5.3)]),
                (4, vec![(rhs_name, 5.3)]),
                (10, vec![(rhs_name, 5.3)]),
                (13, vec![(rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let out: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (3, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (10, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (13, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }

        #[test]
        fn join_ne_ends() -> GenResult<()> {
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            let lhs_name = "LHS";
            let lhs: GenTimeSeries<i32, &str, f64> = vec![
                (0, vec![(lhs_name, 10.3)]),
                (3, vec![(lhs_name, 10.3)]),
                (10, vec![(lhs_name, 10.3)]),
                (11, vec![(lhs_name, 10.3)]),
                (12, vec![(lhs_name, 10.3)]),
            ]
            .into_iter()
            .collect();
            let rhs_name = "RHS";
            let rhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(rhs_name, 5.3)]),
                (3, vec![(rhs_name, 5.3)]),
                (4, vec![(rhs_name, 5.3)]),
                (10, vec![(rhs_name, 5.3)]),
                (13, vec![(rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let out: GenTimeSeries<i32, &str, f64> = vec![
                (3, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
                (10, vec![(lhs_name, 10.3), (rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }

        #[test]
        fn join_empty_lhs() -> GenResult<()> {
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            let lhs_name = "LHS";
            let lhs: GenTimeSeries<i32, &str, f64> = vec![
                (0, vec![(lhs_name, 10.3)]),
                (3, vec![(lhs_name, 10.3)]),
                (10, vec![(lhs_name, 10.3)]),
                (11, vec![(lhs_name, 10.3)]),
                (12, vec![(lhs_name, 10.3)]),
            ]
            .into_iter()
            .collect();
            let rhs: GenTimeSeries<i32, &str, f64> = GenTimeSeries::empty();
            let out: GenTimeSeries<i32, &str, f64> = GenTimeSeries::empty();
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }

        #[test]
        fn join_empty_rhs() -> GenResult<()> {
            let name = format!("join({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
            let lhs: GenTimeSeries<i32, &str, f64> = GenTimeSeries::empty();
            let rhs_name = "RHS";
            let rhs: GenTimeSeries<i32, &str, f64> = vec![
                (1, vec![(rhs_name, 5.3)]),
                (3, vec![(rhs_name, 5.3)]),
                (4, vec![(rhs_name, 5.3)]),
                (10, vec![(rhs_name, 5.3)]),
                (13, vec![(rhs_name, 5.3)]),
            ]
            .into_iter()
            .collect();
            let out: GenTimeSeries<i32, &str, f64> = GenTimeSeries::empty();
            assert_eq!(out.with_name(name.as_str()), lhs.join(rhs)?);
            Ok(())
        }
    }

    #[test]
    fn select() -> GenResult<()> {
        let selector = "close";
        let two_dim_time_series: GenTimeSeries<i32, &str, f64> = vec![
            (3, vec![(selector, 10.3), ("other", 5.3)]),
            (10, vec![(selector, 10.3), ("other", 5.3)]),
            (13, vec![(selector, 10.3), ("other", 5.3)]),
        ]
        .into_iter()
        .collect();
        let expected: GenTimeSeries<i32, &str, f64> = vec![
            (3, vec![(selector, 10.3)]),
            (10, vec![(selector, 10.3)]),
            (13, vec![(selector, 10.3)]),
        ]
        .into_iter()
        .collect();
        let selected = two_dim_time_series.clone().select(selector)?;
        assert_eq!(expected, selected);
        Ok(())
    }

    #[test]
    fn arith_inconsistent_key() -> GenResult<()> {
        let name = format!("add({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
        let lhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![("NOT DEFAULT_INNER_KEY", 12.3)]),
            (3, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (10, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (11, vec![(DEFAULT_INNER_KEY, 12.2)]),
        ]
        .into_iter()
        .collect();
        let rhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (3, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (10, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (11, vec![(DEFAULT_INNER_KEY, 12.4)]),
        ]
        .into_iter()
        .collect();

        Ok(())
    }

    #[test]
    fn add_sub_mul_div() -> GenResult<()> {
        let name = format!("add({},{})", DEFAULT_INNER_KEY, DEFAULT_INNER_KEY);
        let lhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (3, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (10, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (11, vec![(DEFAULT_INNER_KEY, 12.2)]),
        ]
        .into_iter()
        .collect();
        let rhs: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (3, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (10, vec![(DEFAULT_INNER_KEY, 12.3)]),
            (11, vec![(DEFAULT_INNER_KEY, 12.4)]),
        ]
        .into_iter()
        .collect();
        let out: GenTimeSeries<i32, &str, f64> = vec![
            (1, vec![(DEFAULT_INNER_KEY, 24.6)]),
            (3, vec![(DEFAULT_INNER_KEY, 24.6)]),
            (10, vec![(DEFAULT_INNER_KEY, 24.6)]),
            (11, vec![(DEFAULT_INNER_KEY, 24.6)]),
        ]
        .into_iter()
        .collect();
        assert_eq!(out.clone().with_name(&name), lhs.clone().add(rhs.clone())?);
        assert_eq!(
            lhs.clone().with_name(&name),
            lhs.clone()
                .add(lhs.clone())?
                .sub(lhs.clone())?
                .with_name(&name)
        );
        assert_eq!(
            lhs.clone().with_name(&name),
            lhs.clone()
                .mul(lhs.clone())?
                .div(lhs.clone())?
                .with_name(&name)
        );
        Ok(())
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
