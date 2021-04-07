use crate::data::Symbol;
use crate::dto::strategy::TimeSeriesName;
use core::fmt;
use std::fmt::Formatter;

// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html
pub type GenError = Box<dyn std::error::Error>;
pub type GenResult<T> = std::result::Result<T, GenError>;

#[derive(Debug, Clone)]
pub struct UpstreamNotFoundError {
    upstream_name: TimeSeriesName,
}

impl UpstreamNotFoundError {
    pub fn new(upstream_name: TimeSeriesName) -> Box<Self> {
        Box::new(UpstreamNotFoundError { upstream_name })
    }
}

impl fmt::Display for UpstreamNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "upstream not found {}\n", self.upstream_name)
    }
}

impl std::error::Error for UpstreamNotFoundError {
    fn description(&self) -> &str {
        "Upstream not found."
    }
}

#[derive(Debug, Clone)]
pub struct AssetNotFoundError {
    symbol: Symbol,
}

impl AssetNotFoundError {
    pub fn new(symbol: Symbol) -> Box<Self> {
        Box::new(AssetNotFoundError { symbol })
    }
}

impl fmt::Display for AssetNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "asset with symbol {} not found\n", self.symbol)
    }
}

impl std::error::Error for AssetNotFoundError {
    fn description(&self) -> &str {
        "Asset not found."
    }
}

#[derive(Debug, Clone)]
pub struct InvalidStrategyError {
    strategy_name: String,
    reason: String,
}

impl InvalidStrategyError {
    pub fn new(strategy_name: String, reason: String) -> Box<Self> {
        Box::new(InvalidStrategyError {
            strategy_name,
            reason,
        })
    }
}

impl fmt::Display for InvalidStrategyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Invalid strategy: {}\n{}\n",
            self.strategy_name, self.reason
        )
    }
}

impl std::error::Error for InvalidStrategyError {
    fn description(&self) -> &str {
        "Invalid strategy"
    }
}

#[derive(Debug, Clone)]
pub struct TimeSeriesError {
    reason: String,
}

impl TimeSeriesError {
    pub fn new(reason: String) -> Box<Self> {
        Box::new(TimeSeriesError { reason })
    }
}

impl fmt::Display for TimeSeriesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "TimeSeriesError: {}", self.reason)
    }
}

impl std::error::Error for TimeSeriesError {
    fn description(&self) -> &str {
        "Invalid strategy"
    }
}
