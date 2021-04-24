use crate::bot::asset_score::CalculationStatus::Error;
use crate::bot::asset_score::{AssetAllocations, RunnableStrategy};
use crate::data::{Asset, DataClient, Query, QueryType};
use crate::dto::strategy::StrategyDto;
use crate::errors::{GenError, GenResult};
use crate::time_series::{Allocation, DataPointValue, TimeSeries1D, TimeStamp};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct BackTestConfig {
    timestamps: Vec<TimeStamp>,
    strategy: StrategyDto,
    data_client: Box<dyn DataClient>,
}

impl BackTestConfig {
    pub fn new(
        timestamps: Vec<TimeStamp>,
        strategy: StrategyDto,
        data_client: Box<dyn DataClient>,
    ) -> Self {
        BackTestConfig {
            timestamps,
            strategy,
            data_client,
        }
    }
    pub fn timestamps(&self) -> &Vec<TimeStamp> {
        &self.timestamps
    }
    pub fn strategy(&self) -> &StrategyDto {
        &self.strategy
    }
    pub fn data_client(&self) -> &Box<dyn DataClient> {
        &self.data_client
    }
}

pub trait BackTest {
    fn compute_allocations(&self) -> GenResult<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>>;
    fn compute_performance(
        &self,
        allocations: Option<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>>,
    ) -> GenResult<TimeSeries1D>;
    fn compute_result(
        &self,
        allocations: Option<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>>,
    ) -> GenResult<BackTestResult>;
}

impl BackTest for BackTestConfig {
    fn compute_allocations(&self) -> GenResult<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>> {
        let runnable_strategy =
            RunnableStrategy::new(self.strategy().clone(), self.data_client().clone())?;
        let mut allocations: BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>> = BTreeMap::new();
        for timestamp in self.timestamps() {
            allocations
                .entry(timestamp.clone())
                .or_insert(runnable_strategy.compute_allocations(timestamp.clone())?);
        }
        Ok(allocations)
    }
    /// - TODO enforce allocations 0 <= a < 1 where 0 means 0% allocation, 1 means 100% allocation (no shorts)
    /// - TODO enforce performance p >= -1 where -1 means 100% loss
    /// - TODO enforce uniform time increments
    fn compute_performance(
        &self,
        allocations: Option<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>>,
    ) -> GenResult<TimeSeries1D> {
        let mut performance: BTreeMap<TimeStamp, DataPointValue> = BTreeMap::new();
        let allocation_by_timestamp = match allocations {
            Some(allocations) => allocations,
            None => self.compute_allocations()?,
        };
        let timestamps: Vec<&TimeStamp> = allocation_by_timestamp.keys().collect();
        for time_index in 1..timestamps.len() {
            let yesterday = timestamps.get(time_index - 1).unwrap();
            let today = timestamps.get(time_index).unwrap();
            let yesterday_allocations = allocation_by_timestamp.get(yesterday).unwrap();
            // println!("today: {:?}", today);

            let mut today_return = 0f64;
            // TODO optimize to skip 0 allocations
            for asset in yesterday_allocations.keys() {
                // println!("asset: {:?}", asset);
                // TODO optimize for only 1 timestamp
                let relative_price_changes = self.data_client().query(
                    asset,
                    today,
                    Some(Query::new(QueryType::RelativePriceChange)),
                )?;
                // println!("relative_price_changes: {:?}", relative_price_changes);
                let asset_price_change = relative_price_changes.get(today).unwrap();
                // println!("asset_price_change: {:?}", asset_price_change);
                let asset_allocation = yesterday_allocations.get(asset).unwrap();
                // println!("asset_allocation: {:?}", asset_allocation);
                let allocation_return = asset_price_change * asset_allocation;
                today_return += allocation_return;
                // println!("allocation_return: {:?}", allocation_return);
            }
            // println!("today_return: {:?}", today_return);
            performance.entry(*today.clone()).or_insert(today_return);
        }
        Ok(TimeSeries1D::new(performance))
    }

    fn compute_result(
        &self,
        allocations: Option<BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>>>,
    ) -> GenResult<BackTestResult> {
        let timestamp = Utc::now();
        let allocations_by_timestamp: BTreeMap<TimeStamp, BTreeMap<Asset, Allocation>> =
            match allocations {
                Some(allocations) => allocations,
                None => self.compute_allocations()?,
            };
        let mut allocation_vec_lkup: BTreeMap<&Asset, Vec<Allocation>> = BTreeMap::new();
        let mut index: Vec<DateTime<Utc>> = Vec::new();
        for (timestamp, asset_allocation) in allocations_by_timestamp.iter() {
            // println!("timestamp: {:?}", timestamp);
            index.push(timestamp.clone());
            // println!("index={:?}", index);
            for (asset, allocation) in asset_allocation {
                // println!("(asset, allocation): {:?}", (asset, allocation));
                allocation_vec_lkup
                    .entry(asset)
                    .and_modify(|asset_allocations| asset_allocations.push(allocation.clone()))
                    .or_insert(vec![allocation.clone()]);
            }
            // println!("allocation_vec_lkup={:?}", allocation_vec_lkup);
        }
        let allocations: BTreeMap<Asset, TimeSeries1D> = allocation_vec_lkup
            .into_iter()
            .map(|(asset, values)| (asset.clone(), TimeSeries1D::from_vec(index.clone(), values)))
            .collect();

        let performance = self.compute_performance(Some(allocations_by_timestamp.clone()))?;
        Ok(BackTestResult::new(
            timestamp,
            self.clone(),
            allocations,
            performance,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct BackTestResult {
    timestamp: DateTime<Utc>,
    back_test_config: BackTestConfig,
    allocations: BTreeMap<Asset, TimeSeries1D>,
    performance: TimeSeries1D,
}

impl BackTestResult {
    pub fn new(
        timestamp: DateTime<Utc>,
        back_test_config: BackTestConfig,
        allocations: BTreeMap<Asset, TimeSeries1D>,
        performance: TimeSeries1D,
    ) -> Self {
        BackTestResult {
            timestamp,
            back_test_config,
            allocations,
            performance,
        }
    }
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
    pub fn back_test_config(&self) -> &BackTestConfig {
        &self.back_test_config
    }
    pub fn allocations(&self) -> &BTreeMap<Asset, TimeSeries1D> {
        &self.allocations
    }
    pub fn performance(&self) -> &TimeSeries1D {
        &self.performance
    }
}

#[cfg(test)]
mod tests {
    use crate::back_test::{BackTest, BackTestConfig};
    use crate::dto::strategy::{from_path, StrategyDto};
    use crate::errors::GenResult;
    use crate::plot::plot_ts_values;
    use crate::simulation::MockDataClient;
    use crate::time_series::{DataPointValue, TimeSeries1D};
    use chrono::{DateTime, Utc};
    use std::env::current_dir;
    use std::path::Path;

    pub fn get_strategy() -> StrategyDto {
        let strategy_path = current_dir()
            .expect("unable to get working directory")
            .join(Path::new("strategy.yaml"));
        from_path(strategy_path.as_path()).expect("unable to load strategy from path")
    }

    #[test]
    fn back_test() -> GenResult<()> {
        let today = MockDataClient::today();
        let back_test_len: usize = 3;
        let back_test_days: Vec<DateTime<Utc>> = (0..back_test_len)
            .map(|i| today - TimeSeries1D::index_unit() * (back_test_len - i) as i32)
            .collect();
        // println!("back_test_days={:?}", back_test_days);
        let back_test = BackTestConfig::new(
            back_test_days,
            get_strategy(),
            Box::new(MockDataClient::new()),
        );
        let allocations = back_test.compute_allocations()?;
        // println!("allocations={:?}", allocations);
        assert!(!allocations.is_empty());
        let performance = back_test.compute_performance(Some(allocations.clone()))?;
        // println!("performance={:?}", performance);
        assert_eq!(back_test_len - 1, performance.len());
        // TODO expose total_return in api
        let total_return: DataPointValue =
            1.0f64 + performance.add(1.0).values().iter().product::<f64>();
        // println!("total_return={:?}", total_return);
        plot_ts_values(vec![performance]);
        let back_test_result = back_test.compute_result(Some(allocations))?;
        // println!("{:?}", back_test_result);
        assert!(!back_test_result.allocations().is_empty());
        assert_eq!(back_test_len - 1, back_test_result.performance().len());
        Ok(())
    }
}
