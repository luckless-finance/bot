use std::array::IntoIter;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::iter::FromIterator;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use futures::executor;
use grpc::{ClientConf, ClientStubExt};
use protobuf::well_known_types::Timestamp;
use protobuf::SingularPtrField;

use crate::bot::asset_score::CalculationStatus::Error;
use crate::data::{Asset, DataClient, Query, Symbol};
use crate::dto::strategy::{from_path, StrategyDto};
use crate::errors::{GenError, GenResult, QueryError};
use crate::query::{DataPoint, RangedRequest};
use crate::query_grpc::MarketDataClient;
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};

pub struct QueryClient {
    market_data_client: MarketDataClient,
    assets: HashMap<Symbol, Asset>,
}

impl QueryClient {
    pub fn new() -> QueryClient {
        QueryClient {
            market_data_client: build_market_data_client(),
            assets: HashMap::from_iter(IntoIter::new([
                ("A".to_string(), Asset::new("A".to_string())),
                ("B".to_string(), Asset::new("B".to_string())),
            ])),
        }
    }
}

pub fn build_market_data_client() -> MarketDataClient {
    const DEFAULT_PORT: u16 = 50052;
    const HOST: &str = "localhost";
    println!("Building gRPC client for {}:{:?}", HOST, DEFAULT_PORT);
    MarketDataClient::new_plain(HOST, DEFAULT_PORT, ClientConf::new()).expect("client")
}

impl DataClient for QueryClient {
    fn duplicate(&self) -> Box<dyn DataClient> {
        Box::new(QueryClient {
            market_data_client: build_market_data_client(),
            assets: self.assets.clone(),
        })
    }

    fn assets(&self) -> &HashMap<Symbol, Asset> {
        &self.assets
    }

    fn asset(&self, symbol: &Symbol) -> GenResult<&Asset> {
        match &self.assets.get(symbol) {
            Some(asset) => Ok(asset),
            None => Err(QueryError::new("Asset not found".to_string())),
        }
    }

    fn query(&self, query: Query) -> GenResult<TimeSeries1D> {
        let foo = executor::block_on(async {
            let result = self
                .market_data_client
                .query(grpc::RequestOptions::new(), query.try_into()?)
                .await;
            // if result.is_err() {
            //     println!();
            //     Err(QueryError::new(format!("error connecting query server: {:?}", result.err().unwrap())))
            // } else {
            println!("connected to query server");
            let (_meta, resp) = result.unwrap();
            let mut temp: BTreeMap<TimeStamp, DataPointValue> = BTreeMap::new();
            for data_point in resp.await.unwrap().0.data.iter() {
                let timestamp = data_point.clone().timestamp.unwrap();
                let timestamp: TimeStamp =
                    Utc.timestamp(timestamp.seconds, timestamp.nanos.abs() as u32);
                temp.entry(timestamp).or_insert(data_point.value);
                // println!(
                //     "timestamp: '{}', double: {:?}\n",
                //     timestamp.to_rfc3339(),
                //     data_point.double
                // );
            }
            GenResult::Ok(TimeSeries1D::new(temp))
        });
        match foo {
            Err(e) => Err(QueryError::new(format!("fasdf {:?}", e))),
            Ok(ts) => Ok(ts),
        }
    }
}

// TODO add tests

#[allow(dead_code)]
pub fn parse_strategy_path(arg: &str) -> Result<PathBuf, String> {
    let strategy_path = PathBuf::from(arg);
    if !strategy_path.exists() {
        return Err(format!(
            "File does not exist. Expected yaml strategy file.  Got: {:?}",
            arg
        ));
    }
    if !strategy_path.is_file() {
        return Err(format!(
            "Not a file.  Expected yaml strategy file.  Got: {:?}",
            arg
        ));
    }
    match strategy_path.canonicalize() {
        Ok(absolute_path) => Ok(absolute_path),
        Err(error) => Err(error.to_string()),
    }
}

#[allow(dead_code)]
pub fn parse_strategy_yaml(arg: &str) -> Result<StrategyDto, String> {
    let strategy_path: PathBuf = parse_strategy_path(arg)?;
    match from_path(strategy_path.as_path()) {
        Ok(strategy_dto) => Ok(strategy_dto),
        Err(e) => Err(format!(
            "Unable to parse strategy yaml.  Got: {:?}\n{}",
            arg,
            e.to_string()
        )),
    }
}

#[allow(dead_code)]
pub fn parse_date(arg: &str) -> Result<DateTime<Utc>, String> {
    match DateTime::parse_from_rfc3339(arg) {
        Ok(start) => Ok(DateTime::from(start)),
        Err(e) => Err(format!(
            "Unable to parse start.  Expected RFC3339/ISO8601.  Got: {:?}\n{}",
            arg,
            e.to_string()
        )),
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use chrono::Duration;
    use protobuf::SingularPtrField;

    use crate::data::to_proto;
    use crate::query::RangedRequest;
    use crate::query_client::build_market_data_client;
    use crate::query_grpc::MarketDataClient;
    use crate::time_series::{TimeSeries1D, TimeStamp};
    use futures::executor;
    use grpc::{ClientConf, ClientStubExt};

    fn build_request() -> RangedRequest {
        let mut request = RangedRequest::new();
        let now_pb = to_proto(Utc::now());

        request.symbol = "RUST".to_string();
        request.series = "CLOSE".to_string();
        request.first = SingularPtrField::some(now_pb.clone());
        request.last = SingularPtrField::some(now_pb);
        request
    }

    pub async fn query_server(client: &MarketDataClient) {
        println!("query server non-stream");
        let result = client
            .query(grpc::RequestOptions::new(), build_request())
            .await;
        if result.is_err() {
            println!("error connecting query server: {:?}", result.err().unwrap());
        } else {
            println!("connected to query server");
            let (_meta, resp) = result.unwrap();
            for data_point in resp.await.unwrap().0.data.iter() {
                let timestamp = data_point.clone().timestamp.unwrap();
                let timestamp: TimeStamp =
                    Utc.timestamp(timestamp.seconds, timestamp.nanos.abs() as u32);
                println!(
                    "timestamp: '{}', double: {:?}\n",
                    timestamp.to_rfc3339(),
                    data_point.value
                );
            }
        }
    }
    const DEFAULT_PORT: u16 = 50052;
    const HOST: &str = "localhost";

    #[test]
    fn query_single_data_point() {
        println!("gRPC client connecting to {}:{:?}", HOST, DEFAULT_PORT);
        let client =
            MarketDataClient::new_plain(HOST, DEFAULT_PORT, ClientConf::new()).expect("client");
        executor::block_on(async { query_server(&client).await });
    }

    #[test]
    fn server_grpc_server() {
        let client = build_market_data_client();
        query_server(&client);
    }
}
