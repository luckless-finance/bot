#![allow(dead_code)]

use std::cmp::Ordering;
use std::ops::Div;

pub type DataPointValue = f64;
pub type TimeStamp = usize;
pub type Index = Vec<TimeStamp>;

#[derive(Clone, Debug, PartialEq)]
pub struct TimeSeries1D {
    index: Index,
    values: Vec<DataPointValue>,
}

impl TimeSeries1D {
    pub fn from_values(values: Vec<DataPointValue>) -> Self {
        TimeSeries1D::new((0..values.len()).map(|x| x as TimeStamp).collect(), values)
    }
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
    pub fn values(&self) -> &Vec<DataPointValue> {
        &self.values
    }
    pub(crate) fn index(&self) -> &Index {
        &self.index
    }
    fn sma(&self, window_size: usize) -> Self {
        let mut index = self.index.clone();
        index.truncate(self.len() - window_size + 1);
        println!("index={:?}", index);
        let values = self
            .values
            .windows(window_size)
            .map(|x| x.iter().sum::<DataPointValue>())
            .map(|x| x.div(window_size as DataPointValue))
            .collect();
        println!("values={:?}", values);
        TimeSeries1D::new(index, values)
    }
    fn align(&self, rhs: TimeSeries1D) -> (Self, Self) {
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
            TimeSeries1D::new(both_ri, both_l),
            TimeSeries1D::new(both_li, both_r),
        )
    }
    fn scalar_mul(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index.clone();
        let product_values: Vec<DataPointValue> = self.values.iter().map(|x| x * rhs).collect();
        TimeSeries1D::new(product_idx, product_values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    fn mul(&self, rhs: TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.align(rhs);
        println!("{:?}", lhs);
        println!("{:?}", rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l *= *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    fn scalar_add(&self, rhs: DataPointValue) -> Self {
        let product_idx: Index = self.index.clone();
        let product_values: Vec<DataPointValue> = self.values.iter().map(|x| x + rhs).collect();
        TimeSeries1D::new(product_idx, product_values)
    }
    // taken from https://stackoverflow.com/a/53825685
    // generic solution https://stackoverflow.com/a/41207820
    fn add(&self, rhs: TimeSeries1D) -> Self {
        let (mut lhs, rhs) = self.align(rhs);
        for (l, r) in lhs.values.iter_mut().zip(&rhs.values) {
            *l += *r;
        }
        TimeSeries1D::new(lhs.index, lhs.values)
    }
    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::time_series::TimeSeries1D;

    #[test]
    fn new() {
        let values = vec![5., 10., 15.];
        let index = vec![1, 3, 4];
        let ts = TimeSeries1D::new(index, values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values, &[5., 10., 15.]);
        assert_eq!(ts.index, &[1, 3, 4]);
    }

    #[test]
    fn from_values() {
        let values = vec![5., 10., 15.];
        let ts = TimeSeries1D::from_values(values);

        assert_eq!(ts.len(), 3);
        assert_eq!(ts.values, &[5., 10., 15.]);
        assert_eq!(ts.index, &[0, 1, 2]);
    }

    #[test]
    fn sma2() {
        let ts = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let sma = ts.sma(2);

        assert_eq!(sma.len(), 5);
        assert_eq!(sma.values, &[1.5, 2.5, 3.5, 4.5, 6.5]);
        assert_eq!(sma.index, &[2, 4, 5, 6, 7]);
    }

    #[test]
    fn align() {
        let l_in = TimeSeries1D {
            index: vec![2, 4, 6, 7, 8, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let r_in = TimeSeries1D {
            index: vec![1, 2, 4, 5, 7],
            values: vec![1., 2., 3., 4., 8.],
        };
        let (l_out, r_out) = l_in.align(r_in);

        assert_eq!(r_out.index, &[2, 4, 7]);
        assert_eq!(l_out.index, &[2, 4, 7]);
        assert_eq!(l_out.values, &[1., 2., 4.]);
        assert_eq!(r_out.values, &[2., 3., 8.]);
    }

    #[test]
    fn scalar_mul() {
        let ts = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let expected = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![2., 4., 6., 8., 10., 16.],
        };
        let actual = ts.scalar_mul(2.);
        assert_eq!(actual, expected)
    }

    #[test]
    fn mul() {
        let lhs = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let rhs = TimeSeries1D {
            index: vec![1, 4, 5, 7, 9],
            values: vec![1., 2., 3., 4., 5.],
        };
        let expected = TimeSeries1D {
            index: vec![4, 5, 7, 9],
            values: vec![4., 9., 20., 40.],
        };
        let actual = lhs.mul(rhs);
        assert_eq!(actual, expected)
    }

    #[test]
    fn scalar_add() {
        let ts = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let expected = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![-1., 0., 1., 2., 3., 6.],
        };
        let actual = ts.scalar_add(-2.);
        assert_eq!(actual, expected)
    }

    #[test]
    fn add() {
        let lhs = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let rhs = TimeSeries1D {
            index: vec![1, 4, 5, 7, 9],
            values: vec![1., 2., 3., 4., 5.],
        };
        let expected = TimeSeries1D {
            index: vec![4, 5, 7, 9],
            values: vec![4., 6., 9., 13.],
        };
        let actual = lhs.add(rhs.clone());
        let actual2 = rhs.add(lhs);
        assert_eq!(actual, actual2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn len() {
        let ts = TimeSeries1D {
            index: vec![2, 4, 5, 6, 7, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        assert_eq!(ts.len(), 6);
    }
}
