use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::env::current_dir;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::bot::asset_score::{AssetAllocations, AssetScore, RunnableStrategy};
use crate::data::{Asset, DataClient};
use crate::dto::strategy::{from_path, StrategyDto};
use crate::errors::GenResult;
use crate::simulation::MockDataClient;
use crate::time_series::{apply, DataPointValue, TimeSeries1D, TimeStamp};

pub struct Controller {
    runnable_strategy: RunnableStrategy,
    data_client: Box<dyn DataClient>,
}

impl Controller {
    pub fn new(runnable_strategy: RunnableStrategy, data_client: Box<dyn DataClient>) -> Self {
        Controller {
            runnable_strategy,
            data_client,
        }
    }
    pub fn compute_asset_allocation(&self, timestamp: TimeStamp) -> GenResult<AssetAllocations> {
        let runnable_strategy = self.runnable_strategy.duplicate()?;
        let assets: Vec<Asset> = self.data_client.assets().values().cloned().collect();
        let asset_scores: Vec<AssetScore> = assets
            .iter()
            .flat_map(|a| runnable_strategy.run_on_asset(a.clone(), timestamp))
            .collect();
        // determine weightings of all assets in market
        let zeroed: HashMap<Asset, TimeSeries1D> = asset_scores
            .iter()
            .map(|asset_score| {
                (
                    asset_score.asset().clone(),
                    asset_score.score().zero_negatives(),
                )
            })
            .collect();

        // daily sums
        let ts_sum = apply(zeroed.values().collect(), |values| values.iter().sum());
        // allocations
        let weightings: BTreeMap<Asset, DataPointValue> = zeroed
            .iter()
            .map(|asset_score| {
                (
                    asset_score.0.clone(),
                    asset_score
                        .1
                        .ts_div(&ts_sum)
                        .values()
                        .last()
                        .unwrap()
                        .clone(),
                )
            })
            .collect();
        GenResult::Ok(AssetAllocations::new(timestamp, weightings))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::env::current_dir;
    use std::ops::Sub;
    use std::path::Path;

    use chrono::FixedOffset;

    use crate::bot::asset_score::RunnableStrategy;
    use crate::controller::Controller;
    use crate::data::{Asset, DataClient};
    use crate::dto::strategy::{from_path, StrategyDto};
    use crate::errors::GenResult;
    use crate::simulation::{MockDataClient, DATA_SIZE};
    use crate::time_series::TimeSeries1D;

    pub fn get_strategy() -> StrategyDto {
        let strategy_path = current_dir()
            .expect("unable to get working directory")
            .join(Path::new("strategy.yaml"));
        from_path(strategy_path.as_path()).expect("unable to load strategy from path")
    }

    #[test]
    fn three_day_back_test() -> GenResult<()> {
        const BACK_TEST_DURATION: i32 = 3;
        let data_client: Box<dyn DataClient> = Box::new(MockDataClient::new());
        let runnable_strategy = RunnableStrategy::new(get_strategy(), data_client.duplicate())?;
        let controller = Controller::new(runnable_strategy, data_client);
        let start = MockDataClient::today() - TimeSeries1D::index_unit() * BACK_TEST_DURATION;
        for offset in 0..BACK_TEST_DURATION {
            let today = start + TimeSeries1D::index_unit() * offset;
            let result = controller.compute_asset_allocation(today)?;
            println!("{:?}", result);
            assert_eq!(result.allocations().len(), 3);
            assert!(result.allocations().contains_key(&"A".try_into()?));
            assert!(result.allocations().contains_key(&"B".try_into()?));
            assert!(result.allocations().contains_key(&"C".try_into()?));
        }
        Ok(())
    }
}
