#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate luckless;

use std::env::current_dir;
use std::path::Path;

use luckless::bot::asset_score::*;
use luckless::data;
use luckless::dto::strategy::*;
use luckless::simulation::MockDataClient;
use luckless::time_series;

pub fn get_strategy() -> StrategyDto {
    let strategy_path = current_dir()
        .expect("unable to get working directory")
        .join(Path::new("strategy.yaml"));
    from_path(strategy_path.as_path()).expect("unable to load strategy from path")
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::convert::TryInto;

    use chrono::{DateTime, Utc};

    use luckless::bot::asset_score::*;
    use luckless::data::{Asset, DataClient, Query};
    use luckless::dto::strategy::{
        CalculationDto, OperandDto, OperandType, Operation, QueryCalculationDto,
    };
    use luckless::errors::GenResult;
    use luckless::plot::{plot_ts, plot_ts_values};
    use luckless::simulation::MockDataClient;
    use luckless::time_series::{DataPointValue, TimeSeries1D};

    use crate::get_strategy;
    use luckless::query_grpc::MarketDataClient;

    #[test]
    fn plot_asset_prices() -> GenResult<()> {
        let query: Option<QueryCalculationDto> = Some(
            CalculationDto::new(
                "price".to_string(),
                Operation::QUERY,
                vec![OperandDto::new(
                    "field".to_string(),
                    OperandType::Text,
                    "close".to_string(),
                )],
            )
            .try_into()?,
        );
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let asset_price_time_series: Vec<TimeSeries1D> = data_client
            .assets()
            .values()
            .flat_map(|a| data_client.query(a.clone().try_into()?))
            .collect();
        plot_ts_values(asset_price_time_series);
        Ok(())
    }

    // /// Executes a strategy over time
    // ///
    // /// 1. Build runnable strategy
    // /// 2. Replay strategy against historical data
    // ///
    // /// Mock Market contains Assets A, B, C
    // #[test]
    // fn back_test() -> GenResult<()> {
    //     // 1. Build runnable strategy
    //     // load strategy yaml config
    //     let strategy = get_strategy();
    //     // init data client
    //     let data_client: Box<dyn DataClient> = Box::new(MarketDataClient::new());
    //     // build executable strategy
    //     let runnable_strategy = RunnableStrategy::new(strategy, data_client.clone())?;
    //
    //     // 2. Replay strategy against historical data
    //     let back_test_days = 3;
    //     let back_test_end = MockDataClient::today();
    //     let back_test_start = back_test_end - TimeSeries1D::index_unit() * back_test_days;
    //     for time_index in 0..back_test_days {
    //         let today = back_test_start + TimeSeries1D::index_unit() * time_index;
    //         let allocations = runnable_strategy.compute_allocations(today)?;
    //
    //         assert_eq!(allocations.len(), data_client.assets().len())
    //     }
    //
    //     Ok(())
    // }
}
