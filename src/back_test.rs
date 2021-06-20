use crate::bot::asset_score::CalculationStatus::Error;
use crate::bot::asset_score::{AssetAllocations, RunnableStrategy};
use crate::data::{Asset, DataClient, Query, QueryType, Symbol};
use crate::dto::strategy::StrategyDto;
use crate::errors::{GenError, GenResult};
use crate::time_series::{Allocation, DataPointValue, TimeSeries1D, TimeStamp};
use chrono::{DateTime, Utc};
use itertools::zip;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct BackTestConfig {
    index: Vec<TimeStamp>,
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
            index: timestamps,
            strategy,
            data_client,
        }
    }
    pub fn index(&self) -> &Vec<TimeStamp> {
        &self.index
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
        for timestamp in self.index() {
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
                    Query::new(QueryType::RelativePriceChange),
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
        let mut index: Vec<TimeStamp> = Vec::new();
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
        let allocations: BTreeMap<Symbol, TimeSeries1D> = allocation_vec_lkup
            .into_iter()
            .map(|(asset, values)| {
                (
                    asset.symbol().to_string(),
                    TimeSeries1D::from_vec(index.clone(), values),
                )
            })
            .collect();

        let performance = self.compute_performance(Some(allocations_by_timestamp.clone()))?;
        Ok(BackTestResult::new(
            timestamp,
            self.index().clone(),
            self.strategy().clone(),
            allocations,
            performance,
        ))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BackTestResult {
    timestamp: TimeStamp,
    index: Vec<TimeStamp>,
    strategy: StrategyDto,
    allocations: BTreeMap<Symbol, TimeSeries1D>,
    performance: TimeSeries1D,
}

impl BackTestResult {
    pub fn new(
        timestamp: TimeStamp,
        index: Vec<TimeStamp>,
        strategy: StrategyDto,
        allocations: BTreeMap<Symbol, TimeSeries1D>,
        performance: TimeSeries1D,
    ) -> Self {
        BackTestResult {
            timestamp,
            index,
            strategy,
            allocations,
            performance,
        }
    }
    pub fn timestamp(&self) -> TimeStamp {
        self.timestamp
    }
    pub fn index(&self) -> &Vec<TimeStamp> {
        &self.index
    }
    pub fn strategy(&self) -> &StrategyDto {
        &self.strategy
    }
    pub fn allocations(&self) -> &BTreeMap<Symbol, TimeSeries1D> {
        &self.allocations
    }
    pub fn performance(&self) -> &TimeSeries1D {
        &self.performance
    }
}

pub fn dump_result(back_test_result: &BackTestResult) {
    let output_root = Path::new("./output").join(back_test_result.timestamp.to_rfc3339());
    match create_dir_all(output_root.clone()) {
        Ok(_) => (),
        Err(_) => panic!(
            "ERROR unable to create output directory {:?}",
            output_root.to_str()
        ),
    }
    let allocation_path = output_root.join("back_test_result.json");
    let allocation_file = match File::create(allocation_path.clone()) {
        Ok(allocation_file) => allocation_file,
        Err(_) => panic!(
            "ERROR unable to create result file {:?}",
            allocation_path.to_str()
        ),
    };
    match serde_json::to_writer_pretty(&allocation_file, &back_test_result) {
        Ok(_) => println!("OK back test result dumped to: {:?}", allocation_path),
        Err(e) => panic!(
            "ERROR unable to dump back_test_result {:?}\n{}",
            output_root.to_str(),
            e.to_string()
        ),
    };
}

#[cfg(test)]
mod tests {
    use crate::back_test::{dump_result, BackTest, BackTestConfig};
    use crate::data::{Asset, Symbol};
    use crate::dto::strategy::{from_path, StrategyDto};
    use crate::errors::GenResult;
    use crate::plot::plot_ts_values;
    use crate::simulation::MockDataClient;
    use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
    use chrono::{DateTime, Utc};
    use itertools::zip;
    use std::collections::HashMap;
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
        let back_test_days: Vec<TimeStamp> = (0..back_test_len)
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
        let _total_return: DataPointValue =
            1.0f64 + performance.add(1.0).values().iter().product::<f64>();
        // println!("total_return={:?}", total_return);
        plot_ts_values(vec![performance]);
        let back_test_result = back_test.compute_result(Some(allocations))?;
        assert!(!back_test_result.allocations().is_empty());
        assert_eq!(back_test_len - 1, back_test_result.performance().len());

        dump_result(&back_test_result);
        let back_test_result_json = serde_json::to_string(&back_test_result).unwrap();
        println!("{}", back_test_result_json);

        Ok(())
    }
}
