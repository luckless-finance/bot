#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::f64::consts::PI;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Index;

use chrono::{DateTime, TimeZone, Utc};
use gnuplot::AutoOption::Fix;
use gnuplot::Coordinate::Graph;
use gnuplot::PlotOption::Caption;
use gnuplot::{AxesCommon, Figure};
use protobuf::SingularPtrField;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::dto::strategy::QueryCalculationDto;
use crate::errors::{GenError, GenResult};
use crate::query::RangedRequest;
use crate::query_demo::to_proto;
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};

pub type Symbol = String;
pub type Series = String;

pub static DEFAULT_SERIES: &'static str = "DEFAULT";

// TODO allocates on every call
pub fn epoch() -> TimeStamp {
    Utc.ymd(1970, 1, 1).and_hms(0, 0, 0)
}

// TODO allocates on every call
pub fn doomsday() -> TimeStamp {
    Utc.ymd(2050, 1, 1).and_hms(0, 0, 0)
}

pub struct Query {
    symbol: Symbol,
    series: Series,
    first: TimeStamp,
    last: TimeStamp,
}

impl Query {
    pub fn new(symbol: Symbol, series: Series, first: TimeStamp, last: TimeStamp) -> Self {
        Query {
            symbol,
            series,
            first,
            last,
        }
    }
    pub fn complete(symbol: Symbol, series: Series) -> Self {
        Query {
            symbol,
            series,
            first: epoch(),
            last: doomsday(),
        }
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn series(&self) -> &str {
        &self.series
    }
    pub fn first(&self) -> TimeStamp {
        self.first
    }
    pub fn last(&self) -> TimeStamp {
        self.last
    }
}

impl TryFrom<QueryCalculationDto> for Query {
    type Error = GenError;
    fn try_from(_query_calculation_dto: QueryCalculationDto) -> GenResult<Self> {
        GenResult::Ok(Query::complete(
            _query_calculation_dto.name().to_string(),
            _query_calculation_dto.field().to_string(),
        ))
    }
}

impl TryFrom<&Asset> for Query {
    type Error = GenError;

    fn try_from(asset: &Asset) -> GenResult<Self> {
        GenResult::Ok(Query::complete(
            asset.symbol.clone(),
            DEFAULT_SERIES.to_string(),
        ))
    }
}

impl TryFrom<Asset> for Query {
    type Error = GenError;

    fn try_from(asset: Asset) -> GenResult<Self> {
        GenResult::Ok(Query::complete(asset.symbol, DEFAULT_SERIES.to_string()))
    }
}

impl TryFrom<Query> for RangedRequest {
    type Error = GenError;

    fn try_from(query: Query) -> GenResult<Self> {
        let mut request = RangedRequest::new();
        request.symbol = query.symbol;
        request.symbol = query.series;
        request.first = SingularPtrField::some(to_proto(query.first));
        request.last = SingularPtrField::some(to_proto(query.last));
        GenResult::Ok(request)
    }
}

// TODO query memoization/caching
pub trait DataClient {
    fn duplicate(&self) -> Box<dyn DataClient>;
    fn assets(&self) -> &HashMap<Symbol, Asset>;
    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset>;
    // TODO encapsulate params in struct
    // TODO support date ranges to minimize payloads
    fn query(&self, query: Query) -> GenResult<TimeSeries1D>;
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
    use std::convert::TryInto;

    use crate::data::Asset;
    use crate::errors::GenResult;

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
