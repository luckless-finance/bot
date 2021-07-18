#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Neg, Sub};

use chrono::prelude::*;
use chrono::{Duration, TimeZone};
use itertools::{fold, zip};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize, Serializer};

use crate::errors::{GenError, GenResult};

pub type DataPointValue = f64;
// TODO enforce allocations 0 <= a <= 1
pub type Allocation = f64;
pub type TimeStamp = DateTime<Utc>;
pub type Index = Vec<TimeStamp>;

#[derive(Clone, Debug, PartialEq)]
pub struct TimeSeries1D {
    data: BTreeMap<TimeStamp, DataPointValue>,
}

/// Serialize as list of key,value pairs
impl Serialize for TimeSeries1D {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(Some(self.len()))?;
        for (timestamp, value) in &self.data {
            s.serialize_element(&(timestamp, value))?;
        }
        s.end()
    }
}

impl TimeSeries1D {
    // TODO abstraction leak: TimeSeries1D should not know about constraints imposed on time
    pub fn epoch() -> DateTime<Utc> {
        Utc.ymd(2010, 1, 1).and_hms(0, 0, 0)
    }
    // TODO abstraction leak: TimeSeries1D should not know about constraints imposed on time
    pub fn index_unit() -> Duration {
        Duration::days(1)
    }
    pub fn new(data: BTreeMap<TimeStamp, DataPointValue>) -> Self {
        TimeSeries1D { data }
    }
    /// Create new `TimeSeries` from given `index` and `values` vectors
    pub fn from_vec(index: Index, values: Vec<DataPointValue>) -> Self {
        assert_eq!(
            index.len(),
            values.len(),
            "TimeSeries index ({}) and values ({}) must have equal len()",
            index.len(),
            values.len()
        );
        TimeSeries1D {
            data: zip(index, values).collect(),
        }
    }
    /// Create new `TimeSeries` with given `values` and an index containing `(0..values().len())`
    pub fn from_values(values: Vec<DataPointValue>) -> Self {
        TimeSeries1D::from_vec(
            (0..values.len())
                .map(|x| TimeSeries1D::epoch() + Duration::days(x as i64))
                .collect(),
            values,
        )
    }
    /// get clone of `values`
    pub fn values(&self) -> Vec<DataPointValue> {
        self.data.values().cloned().collect()
    }
    /// get clone of `index`
    pub fn index(&self) -> Index {
        self.data.keys().cloned().collect()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn get(&self, timestamp: &TimeStamp) -> Option<&DataPointValue> {
        self.data.get(timestamp)
    }
    /// Align the indices of 2 `TimeSeries`, only values with indices in both `TimeSeries` are included.
    /// Creates 2 new `TimeSeries` instances.
    pub fn intersect(&self, rhs: &TimeSeries1D) -> (Self, Self) {
        let mut l_i = 0;
        let lhs_i = &self.index();
        let lhs_v = &self.values();
        let mut r_i = 0;
        let rhs_i = &rhs.index();
        let rhs_v = &rhs.values();

        let mut both_ri: Index = Vec::new();
        let mut both_li: Index = Vec::new();
        let mut both_r: Vec<DataPointValue> = Vec::new();
        let mut both_l: Vec<DataPointValue> = Vec::new();

        while l_i < lhs_i.len() && r_i < rhs_i.len() {
            match lhs_i[l_i].cmp(&rhs_i[r_i]) {
                Ordering::Less => l_i = l_i + 1,
                Ordering::Equal => {
                    both_li.push(rhs_i[r_i].clone());
                    both_ri.push(rhs_i[r_i].clone());
                    both_r.push(rhs_v[r_i].clone());
                    both_l.push(lhs_v[l_i].clone());
                    l_i = l_i + 1;
                    r_i = r_i + 1;
                }
                Ordering::Greater => r_i = r_i + 1,
            };
        }
        (
            TimeSeries1D::from_vec(both_li, both_l),
            TimeSeries1D::from_vec(both_ri, both_r),
        )
    }
    pub fn add(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index().clone();
        let product_values: Vec<DataPointValue> = self.values().iter().map(|x| x + rhs).collect();
        TimeSeries1D::from_vec(product_idx, product_values)
    }
    pub fn sub(&self, rhs: DataPointValue) -> Self {
        self.add(rhs.neg())
    }
    pub fn mul(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index().clone();
        let product_values: Vec<DataPointValue> = self.values().iter().map(|x| x * rhs).collect();
        TimeSeries1D::from_vec(product_idx, product_values)
    }
    pub fn div(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index().clone();
        let product_values: Vec<DataPointValue> = self.values().iter().map(|x| x / rhs).collect();
        TimeSeries1D::from_vec(product_idx, product_values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_add(&self, rhs: &TimeSeries1D) -> Self {
        let (lhs, rhs) = self.intersect(&rhs);
        let values: Vec<DataPointValue> = zip(lhs.values(), rhs.values())
            .map(|(l, r)| l + r)
            .collect();
        TimeSeries1D::from_vec(lhs.index(), values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_sub(&self, rhs: &TimeSeries1D) -> Self {
        let (lhs, rhs) = self.intersect(&rhs);
        let values: Vec<DataPointValue> = zip(lhs.values(), rhs.values())
            .map(|(l, r)| l - r)
            .collect();
        TimeSeries1D::from_vec(lhs.index(), values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_mul(&self, rhs: &TimeSeries1D) -> Self {
        let (lhs, rhs) = self.intersect(&rhs);
        let values: Vec<DataPointValue> = zip(lhs.values(), rhs.values())
            .map(|(l, r)| l * r)
            .collect();
        TimeSeries1D::from_vec(lhs.index(), values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_div(&self, rhs: &TimeSeries1D) -> Self {
        let (lhs, rhs) = self.intersect(&rhs);
        let values: Vec<DataPointValue> = zip(lhs.values(), rhs.values())
            .map(|(l, r)| l / r)
            .collect();
        TimeSeries1D::from_vec(lhs.index(), values)
    }
    pub fn sma(&self, window_size: usize) -> Self {
        let mut index = self.index().clone();
        index.reverse();
        index.truncate(self.len() - window_size + 1);
        index.reverse();
        let values = self
            .values()
            .windows(window_size)
            .map(|x| x.iter().sum::<DataPointValue>())
            .map(|x| x.div(window_size as DataPointValue))
            .collect();
        TimeSeries1D::from_vec(index, values)
    }
    /// Compute element-wise slope with window length 2
    /// ```text
    ///         final - initial
    /// slope = ───────────────
    ///            initial
    ///```
    /// # Example
    /// ```
    /// use luckless::time_series::TimeSeries1D;
    /// let ts = TimeSeries1D::from_values(vec![1.,4.,3.,4.]);
    /// let slope = ts.slope();
    /// assert_eq!(slope.len(), 3);
    /// assert_eq!(slope.values(), vec![3.,-0.25,1./3.]);
    /// ```
    pub fn slope(&self) -> Self {
        let mut index = self.index().clone();
        index.drain(..1);
        let values = self
            .values()
            .windows(2)
            // TODO handle divide by 0
            .map(|x| (x[1] - x[0]) / x[0])
            .collect();
        TimeSeries1D::from_vec(index, values)
    }

    /// Compute element-wise slope with window length 2
    /// ```text
    ///                    final
    /// relative_change = ───────
    ///                   initial
    ///```
    ///  # Example
    ///  ```
    ///  use luckless::time_series::TimeSeries1D;
    ///  let ts = TimeSeries1D::from_values(vec![1.,4.,3.,1.5]);
    ///  let relative_change = ts.relative_change();
    ///  assert_eq!(relative_change.len(), 3);
    ///  assert_eq!(relative_change.values(), vec![4., 0.75, 0.5]);
    /// /// relative_change can be used to compute overall_change
    ///  let overall_change: f64 = relative_change.values().iter().product();
    ///  assert_eq!(overall_change, 1.5);
    /// /// overall_change can be used to compute the last value from the first
    ///  assert_eq!(ts.values().first().unwrap() * overall_change, *ts.values().last().unwrap());
    /// /// overall_change can be used to compute the first value from the last
    ///  assert_eq!(ts.values().last().unwrap() / overall_change, *ts.values().first().unwrap());
    ///  ```
    pub fn relative_change(&self) -> Self {
        let mut index = self.index().clone();
        index.drain(..1);
        let values = self
            .values()
            .windows(2)
            // TODO handle divide by 0
            .map(|x| x[1] / x[0])
            .collect();
        TimeSeries1D::from_vec(index, values)
    }
    pub fn zero_negatives(&self) -> Self {
        TimeSeries1D::from_vec(
            self.index().clone(),
            self.values()
                .clone()
                .into_iter()
                .map(|value| if value < 0f64 { 0f64 } else { value })
                .collect(),
        )
    }
    pub fn filter_le(&self, timestamp: &TimeStamp) -> Self {
        let tree: BTreeMap<TimeStamp, DataPointValue> = self
            .data
            .range(..=timestamp)
            .map(|(timestamp, value)| (timestamp.clone(), value.clone()))
            .collect();
        TimeSeries1D::new(tree)
    }
    pub fn filter_lt(&self, timestamp: &TimeStamp) -> Self {
        let tree: BTreeMap<TimeStamp, DataPointValue> = self
            .data
            .range(..timestamp)
            .map(|(timestamp, value)| (timestamp.clone(), value.clone()))
            .collect();
        TimeSeries1D::new(tree)
    }
    pub fn filter_ge(&self, timestamp: &TimeStamp) -> Self {
        let tree: BTreeMap<TimeStamp, DataPointValue> = self
            .data
            .range(timestamp..)
            .map(|(timestamp, value)| (timestamp.clone(), value.clone()))
            .collect();
        TimeSeries1D::new(tree)
    }
    pub fn filter_gt(&self, timestamp: &TimeStamp) -> Self {
        let tree: BTreeMap<TimeStamp, DataPointValue> = self
            .data
            .range(timestamp..)
            .skip(1)
            .map(|(timestamp, value)| (timestamp.clone(), value.clone()))
            .collect();
        TimeSeries1D::new(tree)
    }
}

pub fn apply(
    ts_vec: Vec<&TimeSeries1D>,
    func: fn(Vec<DataPointValue>) -> DataPointValue,
) -> TimeSeries1D {
    assert!(ts_vec.len() > 1);
    let index = ts_vec.get(0).unwrap().index().clone();
    let ts_len = index.len();
    TimeSeries1D::from_vec(
        index,
        (0..ts_len)
            .map(|idx| {
                func(
                    ts_vec
                        .iter()
                        .map(|ts| {
                            let values = ts.values();
                            let value = values.get(idx);
                            value.unwrap_or(&0f64).clone()
                        })
                        .collect(),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use chrono::Duration;

    use crate::time_series::TimeSeries1D;

    #[test]
    fn new() {
        let values = vec![5., 10., 15.];
        let index = vec![
            TimeSeries1D::epoch(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
        ];
        let ts = TimeSeries1D::from_vec(index.clone(), values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values(), &[5., 10., 15.]);
        assert_eq!(ts.index(), index);
    }

    #[test]
    fn from_values() {
        let values = vec![5., 10., 15.];
        let ts = TimeSeries1D::from_values(values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values(), &[5., 10., 15.]);
        assert_eq!(
            ts.index(),
            vec![
                TimeSeries1D::epoch(),
                TimeSeries1D::epoch() + Duration::days(1),
                TimeSeries1D::epoch() + Duration::days(2)
            ]
        );
    }

    #[test]
    fn intersect() {
        let l_in = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 8,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5., 8.],
        );
        let r_in = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![1., 2., 3., 4., 8.],
        );
        let expected_l_out = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![1., 2., 4.],
        );
        let expected_r_out = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![2., 3., 8.],
        );
        let (l_out, r_out) = l_in.intersect(&r_in);
        assert_eq!(
            l_out.index(),
            r_out.index(),
            "both TimeSeries should have same indices"
        );
        assert_eq!(l_out.index(), expected_l_out.index());
        assert_eq!(r_out.index(), expected_r_out.index());
        // only keep values with intersecting indices
        assert_eq!(l_out.values(), expected_l_out.values());
        assert_eq!(r_out.values(), expected_r_out.values());
        // high level sanity check
        assert_eq!(l_out, expected_l_out);
        assert_eq!(r_out, expected_r_out);
    }

    #[test]
    fn len() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        assert_eq!(ts.len(), ts.index().len());
        assert_eq!(ts.len(), ts.values().len());
        assert_eq!(ts.len(), 5);
    }

    #[test]
    fn add() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1. + 2., 2. + 2., 3. + 2., 4. + 2., 5. + 2.],
        );
        let actual = ts.add(2.);
        assert_eq!(actual, expected);
        assert_eq!(actual.sub(2.), ts);
    }

    #[test]
    fn sub() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1. - 2., 2. - 2., 3. - 2., 4. - 2., 5. - 2.],
        );
        let actual = ts.sub(2.);
        assert_eq!(actual, expected);
        assert_eq!(actual.add(2.), ts);
    }

    #[test]
    fn mul() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1. * 2., 2. * 2., 3. * 2., 4. * 2., 5. * 2.],
        );
        let actual = ts.mul(2.);
        assert_eq!(actual, expected);
        assert_eq!(actual.div(2.), ts);
    }

    #[test]
    fn div() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1. / 2., 2. / 2., 3. / 2., 4. / 2., 5. / 2.],
        );
        let actual = ts.div(2.);
        assert_eq!(actual, expected);
        assert_eq!(actual.mul(2.), ts);
    }

    #[test]
    fn ts_add() {
        let lhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![
                2. + -5., // -3 @ 4
                3. + 4.,  // 7 @ 5
                4. + 2.,  // 6 @ 7
            ],
        );
        let actual = lhs.ts_add(&rhs);
        assert_eq!(actual, expected);
    }

    #[test]
    fn ts_sub() {
        let lhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![7., -1., 2.],
        );
        let actual = lhs.ts_sub(&rhs);
        assert_eq!(actual, expected);
    }

    #[test]
    fn ts_mul() {
        let lhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-10., 12., 8.],
        );
        let actual = lhs.ts_mul(&rhs);
        assert_eq!(actual, expected);
    }

    #[test]
    fn ts_div() {
        let lhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-0.4, 0.75, 2.],
        );
        let actual = lhs.ts_div(&rhs);
        assert_eq!(actual, expected);
    }

    #[test]
    fn sma() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let sma = ts.sma(2);

        assert_eq!(sma.len(), 4);
        assert_eq!(sma.values(), &[1.5, 2.5, 3.5, 4.5]);
        assert_eq!(
            sma.index(),
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ]
        );
    }

    #[test]
    fn slope() {
        let values = vec![1., 4., 3., 6.];
        let index = vec![
            TimeSeries1D::epoch(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
            TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 3,
        ];
        let ts = TimeSeries1D::from_vec(index.clone(), values).slope();
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values(), &[3., -0.25, 1.]);
    }

    #[test]
    fn zero_negatives() {
        let values = vec![-5., 10., 0.];
        let index = vec![
            TimeSeries1D::epoch(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit(),
            TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
        ];
        let ts = TimeSeries1D::from_vec(index.clone(), values);
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.zero_negatives().values(), &[0., 10., 0.]);
        assert_eq!(ts.index(), index);
    }

    #[test]
    fn filter_le() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![1., 2., 3., 4.],
        );
        let actual = ts.filter_le(&(TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn filter_lt() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
            ],
            vec![1., 2., 3.],
        );
        let actual = ts.filter_lt(&(TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn filter_ge() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![4., 5.],
        );
        let actual = ts.filter_ge(&(TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7));
        assert_eq!(actual, expected);
    }

    #[test]
    fn filter_gt() {
        let ts = TimeSeries1D::from_vec(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::from_vec(
            vec![TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9],
            vec![5.],
        );
        let actual = ts.filter_gt(&(TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7));
        assert_eq!(actual, expected);
    }
}
