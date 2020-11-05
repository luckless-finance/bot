use std::cmp::Ordering;
use std::ops::Div;

use rand::thread_rng;
use rand_distr::{Distribution, Normal};

pub type DataPointValue = f64;

pub fn query(limit: i32) -> Vec<DataPointValue> {
    let mut x: DataPointValue = 100.0;
    let mut values: Vec<DataPointValue> = Vec::new();
    let mut ran = thread_rng();
    let log_normal = Normal::new(0.0, 1.0).unwrap();
    for _i in 0..limit {
        x = log_normal.sample(&mut ran) + x;
        values.push(x);
    }
    values
}

pub fn sma(ts: Vec<DataPointValue>, window_size: usize) -> Vec<DataPointValue> {
    ts.windows(window_size)
        .map(|window| window.iter()
            .sum::<DataPointValue>()
            .div(window_size as DataPointValue))
        .collect()
}

trait TS {
    fn sma(&self, window_size: usize) -> Self;
    fn align(&self, rhs: Self) -> (Self, Self) where Self: std::marker::Sized;
    fn scalar_mul(&self, rhs: DataPointValue) -> Self;
    fn mul(&self, rhs: Self) -> Self;
    fn scalar_sub(&self, rhs: DataPointValue) -> Self;
    fn sub(&self, rhs: Self) -> Self;
    fn len(&self) -> usize;
}

#[derive(Clone, Debug)]
struct TimeSeries {
    index: Vec<i64>,
    values: Vec<DataPointValue>,
}

impl TimeSeries {
    pub fn from_values(values: Vec<DataPointValue>) -> Self {
        TimeSeries { index: (0..values.len()).map(|x| x as i64).collect(), values }
    }
    pub fn new(index: Vec<i64>, values: Vec<DataPointValue>) -> Self {
        TimeSeries { index, values }
    }
}

impl TS for TimeSeries {
    fn sma(&self, window_size: usize) -> Self {
        let mut index = self.index.clone();
        index.truncate(self.len() - window_size + 1);
        println!("index={:?}", index);
        let values = self.values.windows(window_size)
            .map(|x| x.iter().sum::<f64>())
            .map(|x| x.div(window_size as DataPointValue))
            .collect();
        println!("values={:?}", values);
        TimeSeries::new(index, values)
    }

    fn align(&self, rhs: TimeSeries) -> (Self, Self) {
        let mut l_i = 0;
        let mut lhs_i = &self.index;
        let mut lhs_v = &self.values;
        let mut r_i = 0;
        let mut rhs_i = &rhs.index;
        let mut rhs_v = &rhs.values;

        let mut both_ri: Vec<i64> = Vec::new();
        let mut both_li: Vec<i64> = Vec::new();
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
            // println!("{:?}", both_li);
            // println!("{:?}", both_ri);
            // println!("{:?}", both_l);
            // println!("{:?}", both_r);
        }
        (TimeSeries::new(both_ri, both_l), TimeSeries::new(both_li, both_r))
    }

    fn scalar_mul(&self, rhs: f64) -> Self {
        unimplemented!()
    }

    fn mul(&self, rhs: TimeSeries) -> Self {
        unimplemented!()
    }

    fn scalar_sub(&self, rhs: f64) -> Self {
        unimplemented!()
    }

    fn sub(&self, rhs: TimeSeries) -> Self {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.index.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{DataPointValue, query, sma, TimeSeries, TS};

    fn get_timeseries() -> TimeSeries {
        let values = (1..=100).map(|x| x as DataPointValue).collect();
        let ts: TimeSeries = TimeSeries::from_values(values);
        println!("{:?}", ts);
        ts
    }

    #[test]
    fn sma2() {
        let ts = TimeSeries { index: vec![2, 4, 5, 6, 7, 9], values: vec![1., 2., 3., 4., 5., 8.] };
        let sma = ts.sma(2);

        assert_eq!(sma.values, &[1.5, 2.5, 3.5, 4.5, 6.5]);
        assert_eq!(sma.index, &[2, 4, 5, 6, 7]);
        assert_eq!(sma.len(), 5);
    }

    #[test]
    fn align() {
        let l_in = TimeSeries {
            index: vec![2, 4, 6, 7, 8, 9],
            values: vec![1., 2., 3., 4., 5., 8.],
        };
        let r_in = TimeSeries {
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
    fn foo() {
        let ts = query(1000);
        sma(ts, 3);
    }
}
