#![allow(dead_code)]

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Neg};

use chrono::prelude::*;
use chrono::{Duration, TimeZone};

use crate::errors::{GenError, GenResult};

mod core;

pub type DataPointValue = f64;
pub type TimeStamp = DateTime<Utc>;
pub type Index = Vec<TimeStamp>;

#[derive(Clone, Debug, PartialEq)]
pub struct TimeSeries1D {
    index: Index,
    values: Vec<DataPointValue>,
}

impl TimeSeries1D {
    pub fn epoch() -> DateTime<Utc> {
        Utc.ymd(2010, 1, 1).and_hms(0, 0, 0)
    }
    pub fn index_unit() -> Duration {
        Duration::days(1)
    }
    /// Create new `TimeSeries` from given `index` and `values` vectors
    pub fn new(index: Index, values: Vec<DataPointValue>) -> Self {
        assert_eq!(
            index.len(),
            values.len(),
            "TimeSeries index ({}) and values ({}) must have equal len()",
            index.len(),
            values.len()
        );
        TimeSeries1D { index, values }
    }
    /// Create new `TimeSeries` with given `values` and an index containing `(0..values.len())`
    pub fn from_values(values: Vec<DataPointValue>) -> Self {
        TimeSeries1D::new(
            (0..values.len())
                .map(|x| TimeSeries1D::epoch() + Duration::days(x as i64))
                .collect(),
            values,
        )
    }
    /// Borrow `values`
    pub fn values(&self) -> &Vec<DataPointValue> {
        &self.values
    }
    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn len(&self) -> usize {
        self.index.len()
    }
    /// Align the indices of 2 `TimeSeries`, only values with indices in both `TimeSeries` are included.
    /// Creates 2 new `TimeSeries` instances.
    pub fn intersect(&self, rhs: &TimeSeries1D) -> (Self, Self) {
        let mut l_i = 0;
        let lhs_i = &self.index;
        let lhs_v = &self.values;
        let mut r_i = 0;
        let rhs_i = &rhs.index;
        let rhs_v = &rhs.values;

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
            TimeSeries1D::new(both_li, both_l),
            TimeSeries1D::new(both_ri, both_r),
        )
    }
    pub fn add(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index.clone();
        let product_values: Vec<DataPointValue> = self.values.iter().map(|x| x + rhs).collect();
        TimeSeries1D::new(product_idx, product_values)
    }
    pub fn sub(&self, rhs: DataPointValue) -> Self {
        self.add(rhs.neg())
    }
    pub fn mul(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index.clone();
        let x = 5;
        let product_values: Vec<DataPointValue> = self.values.iter().map(|x| x * rhs).collect();
        TimeSeries1D::new(product_idx, product_values)
    }
    pub fn div(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index.clone();
        let product_values: Vec<DataPointValue> = self.values.iter().map(|x| x / rhs).collect();
        TimeSeries1D::new(product_idx, product_values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_add(&self, rhs: &TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.intersect(&rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l += *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_sub(&self, rhs: &TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.intersect(&rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l -= *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_mul(&self, rhs: &TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.intersect(&rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l *= *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    pub fn ts_div(&self, rhs: &TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.intersect(&rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l /= *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    pub fn sma(&self, window_size: usize) -> Self {
        let mut index = self.index.clone();
        index.reverse();
        index.truncate(self.len() - window_size + 1);
        index.reverse();
        let values = self
            .values
            .windows(window_size)
            .map(|x| x.iter().sum::<DataPointValue>())
            .map(|x| x.div(window_size as DataPointValue))
            .collect();
        TimeSeries1D::new(index, values)
    }
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
        let ts = TimeSeries1D::new(index.clone(), values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values, &[5., 10., 15.]);
        assert_eq!(ts.index, index);
    }

    #[test]
    fn from_values() {
        let values = vec![5., 10., 15.];
        let ts = TimeSeries1D::from_values(values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values, &[5., 10., 15.]);
        assert_eq!(
            ts.index,
            vec![
                TimeSeries1D::epoch(),
                TimeSeries1D::epoch() + Duration::days(1),
                TimeSeries1D::epoch() + Duration::days(2)
            ]
        );
    }

    #[test]
    fn intersect() {
        let l_in = TimeSeries1D::new(
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
        let r_in = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![1., 2., 3., 4., 8.],
        );
        let expected_l_out = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![1., 2., 4.],
        );
        let expected_r_out = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 2,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![2., 3., 8.],
        );
        let (l_out, r_out) = l_in.intersect(&r_in);
        assert_eq!(
            l_out.index, r_out.index,
            "both TimeSeries should have same indices"
        );
        assert_eq!(l_out.index, expected_l_out.index);
        assert_eq!(r_out.index, expected_r_out.index);
        // only keep values with intersecting indices
        assert_eq!(l_out.values, expected_l_out.values);
        assert_eq!(r_out.values, expected_r_out.values);
        // high level sanity check
        assert_eq!(l_out, expected_l_out);
        assert_eq!(r_out, expected_r_out);
    }

    #[test]
    fn len() {
        let ts = TimeSeries1D::new(
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
        let ts = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::new(
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
        let ts = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::new(
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
        let ts = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::new(
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
        let ts = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let expected = TimeSeries1D::new(
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
        let lhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-3., 7., 6.],
        );
        let actual = lhs.ts_add(&rhs);
        assert_eq!(actual, expected);
    }

    #[test]
    fn ts_sub() {
        let lhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::new(
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
        let lhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::new(
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
        let lhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 1,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ],
            vec![1., 2., 3., 4., 5.],
        );
        let rhs = TimeSeries1D::new(
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 6,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
            ],
            vec![-5., 4., 3., 2.],
        );
        let expected = TimeSeries1D::new(
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
        let ts = TimeSeries1D::new(
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
        assert_eq!(sma.values, &[1.5, 2.5, 3.5, 4.5]);
        assert_eq!(
            sma.index,
            vec![
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 4,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 5,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 7,
                TimeSeries1D::epoch() + TimeSeries1D::index_unit() * 9,
            ]
        );
    }
}
