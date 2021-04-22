#![allow(dead_code)]

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Index;

use chrono::{DateTime, Utc};
use gnuplot::AutoOption::Fix;
use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};

use crate::dto::strategy::QueryCalculationDto;
use crate::errors::GenResult;
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use std::borrow::Borrow;

pub type Symbol = String;

// TODO query memoization/caching
pub trait DataClient {
    fn assets(&self) -> &HashMap<Symbol, Asset>;
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset>;
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query: Option<QueryCalculationDto>,
    ) -> GenResult<TimeSeries1D>;
}
impl fmt::Debug for dyn DataClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataClient")
            .field("assets", &self.assets().len())
            .finish()
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Asset {
    symbol: Symbol,
}

impl Asset {
    pub fn new(symbol: Symbol) -> Self {
        Asset { symbol }
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
