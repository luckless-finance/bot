#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate yafa;

use std::env::current_dir;
use std::path::Path;

use yafa::bot::asset_score::*;
use yafa::data;
use yafa::dto::strategy::*;
use yafa::simulation::MockDataClient;
use yafa::time_series;

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

    use yafa::bot::asset_score::*;
    use yafa::data::{Asset, DataClient, };
    use yafa::dto::strategy::{CalculationDto, OperandDto, OperandType, Operation, QueryCalculationDto};
    use yafa::errors::GenResult;
    use yafa::simulation::MockDataClient;
    use yafa::time_series::{apply, DataPointValue, TimeSeries1D};

    use crate::get_strategy;
    use yafa::plot::plot_ts;

    #[test]
    fn plot_asset_prices() -> GenResult<()> {
        let query: Option<QueryCalculationDto> = Some(CalculationDto::new(
            "price".to_string(),
            Operation::QUERY,
            vec![
                OperandDto::new("field".to_string(), OperandType::Text, "close".to_string())
            ]).try_into()?);
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let asset_price_time_series: Vec<&TimeSeries1D> = data_client.assets().values()
            .flat_map(|a| data_client.query(
                a,
                &MockDataClient::today(),
                None,
            ))
            .collect();
        plot_ts(asset_price_time_series);
        Ok(())
    }

    /// Mock Market contains Assets A, B, C
    #[test]
    fn back_test() -> GenResult<()> {
        // apply strategy on market (DataClient) to determine score
        let strategy = get_strategy();
        let compiled_strategy = CompiledStrategy::new(strategy)?;
        // symbols: "A", "B" and "C"
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let asset_scores: Vec<AssetScore> = data_client.assets().values()
            .flat_map(|a|
                compiled_strategy.asset_score(a.clone(),
                                              MockDataClient::today(),
                                              Box::new(MockDataClient::new()))
            )
            .collect();
        let asset_score_time_series: HashMap<&Asset, &TimeSeries1D> = asset_scores.iter()
            .map(|asset_score| (asset_score.asset(), asset_score.score()))
            .collect();
        let score_time_series: Vec<&TimeSeries1D> = asset_score_time_series.values().cloned().collect();
        plot_ts(
            score_time_series.clone()
        );

        // determine weightings of all assets in market
        let zeroed: Vec<TimeSeries1D> = score_time_series.iter()
            .map(|score_ts| score_ts.zero_negatives())
            .collect();
        plot_ts(
            zeroed.iter().collect()
        );
        let ts_sum = apply(zeroed.iter().collect(), |values| values.iter().sum());
        let weightings: Vec<TimeSeries1D> = zeroed.iter()
            .map(|score_ts| score_ts.ts_div(&ts_sum))
            .collect();
        plot_ts(
            weightings.iter().collect()
        );


        Ok(())
    }
}

