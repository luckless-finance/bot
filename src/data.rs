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
use crate::errors::{GenError, GenResult};
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::TryFrom;

pub type Symbol = String;

pub enum QueryType {
    AbsolutePrice,
    RelativePriceChange,
}

pub struct Query {
    query_type: QueryType,
}

impl Query {
    pub fn new(query_type: QueryType) -> Self {
        Query { query_type }
    }
    pub fn query_type(&self) -> &QueryType {
        &self.query_type
    }
}

impl TryFrom<QueryCalculationDto> for Query {
    type Error = GenError;
    fn try_from(_query_calculation_dto: QueryCalculationDto) -> GenResult<Self> {
        GenResult::Ok(Query {
            query_type: QueryType::AbsolutePrice,
        })
    }
}

// TODO query memoization/caching
pub trait DataClient {
    fn duplicate(&self) -> Box<dyn DataClient>;
    fn assets(&self) -> &HashMap<Symbol, Asset>;
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset>;
    // TODO encapsulate params in struct
    // TODO support date ranges to minimize payloads
    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        // TODO make required
        query: Option<Query>,
    ) -> GenResult<TimeSeries1D>;
}

impl Clone for Box<dyn DataClient> {
    fn clone(&self) -> Box<dyn DataClient> {
        self.duplicate()
    }
}

impl fmt::Debug for dyn DataClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataClient")
            .field("assets", &self.assets().len())
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, Serialize)]
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

impl TryFrom<&str> for Asset {
    type Error = GenError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        GenResult::Ok(Asset::new(value.to_string()))
    }
}

impl Display for Asset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.symbol.as_str().partial_cmp(other.symbol())
    }
}

#[cfg(test)]
mod tests {
    use crate::data::Asset;
    use crate::errors::GenResult;
    use std::convert::TryInto;

    #[test]
    fn str_to_asset() -> GenResult<()> {
        assert_eq!(Asset::new("A".to_string()), "A".try_into()?);
        GenResult::Ok(())
    }

    #[test]
    fn asset_ordering() -> GenResult<()> {
        let mut unordered: Vec<Asset> = vec![
            "Z".try_into()?,
            "A".try_into()?,
            "AA".try_into()?,
            "ABC".try_into()?,
        ];
        let expected: Vec<Asset> = vec![
            "A".try_into()?,
            "AA".try_into()?,
            "ABC".try_into()?,
            "Z".try_into()?,
        ];
        unordered.sort();
        assert_eq!(expected, unordered);
        GenResult::Ok(())
    }
}
