#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate yafa;

use yafa::bot::asset_score::*;
use yafa::time_series;
use yafa::dto::strategy::*;
use yafa::data;
use yafa::simulation::MockDataClient;
use std::env::current_dir;
use std::path::Path;

pub fn get_strategy() -> StrategyDto {
    let strategy_path = current_dir()
        .expect("unable to get working directory")
        .join(Path::new("strategy.yaml"));
    from_path(strategy_path.as_path()).expect("unable to load strategy from path")
}

#[cfg(test)]
mod tests {
    use crate::get_strategy;
    use yafa::simulation::{MockDataClient};
    use yafa::data::{DataClient, plot_ts};
    use yafa::bot::asset_score::*;
    use yafa::time_series::{DataPointValue, TimeSeries1D};
    use yafa::errors::GenResult;
    use yafa::dto::strategy::{CalculationDto, Operation, QueryCalculationDto, OperandDto, OperandType};
    use std::convert::TryInto;


    #[test]
    fn back_test() -> GenResult<()> {
        let strategy = get_strategy();
        let bot = CompiledStrategy::new(strategy)?;
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let query: Option<QueryCalculationDto> = Some(CalculationDto::new(
            "price".to_string(),
            Operation::QUERY,
            vec![
                OperandDto::new("field".to_string(), OperandType::Text, "close".to_string())
            ]).try_into()?);
        let asset_price_time_series: Vec<&TimeSeries1D> = data_client.assets().values()
            .flat_map(|a| data_client.query(
                a,
                &MockDataClient::today(),
                None,
            ))
            .collect();
        // plot_ts(asset_price_time_series);

        let asset_scores: Vec<AssetScore> = data_client.assets().values()
            .flat_map(|a|
                bot.asset_score(a.clone(),
                                MockDataClient::today(),
                                Box::new(MockDataClient::new()))
            )
            .collect();
        // println!("{:?}", asset_scores);
        let asset_score_time_series: Vec<&TimeSeries1D> = asset_scores.iter()
            .map(|asset_score| asset_score.score())
            .collect();
        // plot_ts(asset_score_time_series);
        Ok(())
    }
}

