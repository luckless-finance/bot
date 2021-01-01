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


    #[test]
    fn back_test() -> GenResult<()> {
        let strategy = get_strategy();
        let bot = Bot::new(strategy.clone())?;
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let mut bots: Vec<AssetScore> = data_client.assets().values()
            .flat_map(|a|
                bot.asset_score(a.clone(),
                                TODAY,
                                Box::new(MockDataClient::new()))
            )
            .collect();
        // bots.iter_mut().for_each(|b| b.execute().unwrap());
        println!("{:?}", bots);
        // let _ts: Vec<&TimeSeries1D> = bots.iter()
        //     .map(|b| b.upstream(strategy.score().calc()).unwrap())
        //     .collect();
        // plot_ts(ts);
        // let scores: Vec<&DataPointValue> = bots.iter()
        //     .flat_map(|b| b.score())
        //     .collect();
        // println!("{:?}", scores);
        Ok(())
        // let b: &mut yafa::bot::ExecutableBot = bots.get_mut(0).unwrap();
        // b.execute()
    }
}

