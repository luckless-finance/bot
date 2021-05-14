use std::collections::{BTreeMap, HashMap};

use chrono::{TimeZone, Utc};

use futures::executor;

use crate::bot::asset_score::CalculationStatus::Error;
use crate::data::{Asset, DataClient, Query, Symbol};
use crate::errors::{GenError, GenResult, QueryError};
use crate::query::{DataPoint, RangeRequest};
use crate::query_grpc::MarketDataClient;
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use protobuf::well_known_types::Timestamp;
use protobuf::SingularPtrField;

pub struct QueryClient {
    market_data_client: MarketDataClient,
    assets: HashMap<Symbol, Asset>,
}

impl DataClient for QueryClient {
    fn duplicate(&self) -> Box<dyn DataClient> {
        todo!("meh")
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

    fn query(
        &self,
        asset: &Asset,
        timestamp: &TimeStamp,
        query: Option<Query>,
    ) -> GenResult<TimeSeries1D> {
        let foo = executor::block_on(async {
            let mut request = RangeRequest::new();
            request.symbol = "A".to_string();
            let mut timestamp_pb = Timestamp::new();

            let result = self
                .market_data_client
                .query(grpc::RequestOptions::new(), request)
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
                temp.entry(timestamp).or_insert(data_point.double);
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
