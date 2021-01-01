#![allow(unused_imports)]
extern crate yafa;

use yafa::bot::*;
use yafa::time_series;
use yafa::strategy::*;
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
    use yafa::simulation::{MockDataClient, TODAY};
    use yafa::data::{DataClient, plot_ts};
    use yafa::bot::*;
    use yafa::time_series::{DataPointValue, TimeSeries1D};
    use yafa::errors::GenResult;
    use yafa::strategy::{CalculationDto, Operation, QueryCalculationDto};
    use std::convert::TryInto;


    #[test]
    fn back_test() -> GenResult<()> {
        let strategy = get_strategy();
        let bot = Bot::new(strategy)?;
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let query: Option<QueryCalculationDto> = Some(CalculationDto::new(
            "price".to_string(),
            Operation::QUERY,
            vec![]).try_into()?);
        // let asset_time_series: Vec<&TimeSeries1D> = data_client.assets().values()
        //     .flat_map(|a| data_client.query(
        //         a,
        //         &TODAY,
        //         None,
        //     ))
        //     .collect();
        // let asset_scores: Vec<AssetScore> = data_client.assets().values()
        //     .flat_map(|a|
        //         bot.asset_score(a.clone(),
        //                         TODAY,
        //                         Box::new(MockDataClient::new()))
        //     )
        //     .collect();
        // // println!("{:?}", asset_scores);
        // let ts: Vec<&TimeSeries1D> = asset_scores.iter()
        //     .map(|asset_score| asset_score.score())
        //     .collect();
        // // plot_ts(ts);
        Ok(())
    }
}

