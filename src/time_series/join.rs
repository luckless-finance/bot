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
use std::collections::hash_map::RandomState;
use std::convert::TryInto;
use std::hash::Hash;

const DEFAULT_KEY: &str = "DEFAULT";
const DEFAULT_TIME_SERIES_NAME: &str = "DEFAULT";

trait Limits {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

trait Merge {
    fn merge(lhs: Self, rhs: Self) -> Self;
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Time {
    time: String,
}

impl TryInto<Time> for i32 {
    type Error = GenError;

    fn try_into(self) -> Result<Time, Self::Error> {
        Ok(Time {
            time: self.to_string(),
        })
    }
}

impl Limits for Time {
    fn min_value() -> Self {
        Time {
            time: "".to_string(),
        }
    }

    fn max_value() -> Self {
        Time {
            time: "ZZZZZZ".to_string(),
        }
    }
}

impl Merge for V {
    fn merge(lhs: Self, rhs: Self) -> Self {
        lhs + rhs
    }
}

// type T = i32;
type T = Time;
type V = i32;

pub fn join(lhs: BTreeMap<T, V>, rhs: BTreeMap<T, V>, merge: fn(V, V) -> V) -> BTreeMap<T, V> {
    let mut lhs_t = lhs.first_key_value().unwrap().0;
    let lhs_tn = lhs.last_key_value().unwrap().0;
    let mut rhs_t = rhs.first_key_value().unwrap().0;
    let rhs_tn = rhs.last_key_value().unwrap().0;

    println!("lhs = {:?}\nlhs_t={:?} lhs_tn={:?}", lhs, lhs_t, lhs_tn);
    println!("rhs = {:?}\nrhs_t={:?} rhs_tn={:?}", rhs, rhs_t, rhs_tn);

    let max_value = T::max_value();

    let mut i = 0;
    let mut out: BTreeMap<T, V> = BTreeMap::new();
    while lhs_t <= lhs_tn && rhs_t <= rhs_tn {
        match lhs_t.cmp(rhs_t) {
            Ordering::Less => {
                println!("\nlhs < rhs , {:?} < {:?}", lhs_t, rhs_t);
                // move lhs to rhs
                lhs_t = lhs.range(rhs_t..).next().map_or(&max_value, |(t, _)| t);
                println!("move so that lhs_t = {:?}", lhs_t);
            }
            Ordering::Equal => {
                println!("\nlhs = rhs , {:?} = {:?}", lhs_t, rhs_t);
                out.insert(lhs_t.clone(), merge(lhs[&lhs_t], rhs[&lhs_t]));
                println!("out = {:?}", out);

                let mut lhs_iter = lhs.range(lhs_t..);
                lhs_iter.advance_by(1);
                lhs_t = lhs_iter.next().map_or(&max_value, |(t, _)| t);

                let mut rhs_iter = rhs.range(rhs_t..);
                rhs_iter.advance_by(1);
                rhs_t = rhs_iter.next().map_or(&max_value, |(t, _)| t);

                println!("move both lhs_t = {:?}; rhs_t = {:?}", lhs_t, rhs_t);
            }
            Ordering::Greater => {
                println!("\nlhs > rhs , {:?} > {:?}", lhs_t, rhs_t);
                // move rhs up to the next largest index after
                rhs_t = rhs.range(lhs_t..).next().map_or(&max_value, |(t, _)| t);
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

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::error::Error;
    use std::ops::Add;

    use chrono::prelude::*;
    use itertools::EitherOrBoth::{Both, Left, Right};

    use crate::errors::GenResult;
    use crate::time_series::join::*;
    use std::borrow::Borrow;

    #[test]
    fn btreemap_join_eq_end() -> GenResult<()> {
        let mut lhs = BTreeMap::new();
        lhs.insert(2.try_into()?, 0);
        lhs.insert(3.try_into()?, 1);
        lhs.insert(10.try_into()?, 3);
        lhs.insert(11.try_into()?, 4);
        lhs.insert(13.try_into()?, 4);
        let mut rhs = BTreeMap::new();
        rhs.insert(1.try_into()?, 0);
        rhs.insert(3.try_into()?, 1);
        rhs.insert(4.try_into()?, 2);
        rhs.insert(10.try_into()?, 3);
        rhs.insert(13.try_into()?, 4);
        let mut out = BTreeMap::new();
        out.insert(3.try_into()?, 2);
        out.insert(10.try_into()?, 6);
        out.insert(13.try_into()?, 8);
        assert_eq!(out, join(lhs, rhs, V::merge));
        Ok(())
    }

    #[test]
    fn btreemap_join_eq_start() -> GenResult<()> {
        let mut lhs = BTreeMap::new();
        lhs.insert(1.try_into()?, 0);
        lhs.insert(3.try_into()?, 1);
        lhs.insert(10.try_into()?, 3);
        lhs.insert(11.try_into()?, 4);
        let mut rhs = BTreeMap::new();
        rhs.insert(1.try_into()?, 0);
        rhs.insert(3.try_into()?, 1);
        rhs.insert(4.try_into()?, 2);
        rhs.insert(10.try_into()?, 3);
        rhs.insert(13.try_into()?, 4);
        let mut out = BTreeMap::new();
        out.insert(3.try_into()?, 2);
        out.insert(10.try_into()?, 6);
        // BTreeMap sorts its keys
        out.insert(1.try_into()?, 0);
        assert_eq!(out, join(lhs, rhs, V::merge));
        Ok(())
    }
}
