#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, DataClient};
use crate::errors::{GenResult, UpstreamNotFoundError};
use crate::strategy::{
    CalculationDto, DyadicScalarCalculationDto, DyadicTsCalculationDto, Operation,
    QueryCalculationDto, SmaCalculationDto, StrategyDto, TimeSeriesName,
};
use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};
use serde::export::Formatter;
use std;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CalculationStatus {
    NotStarted,
    InProgress,
    Complete,
    Error,
}

/// Wraps several Dtos required traverse and consume a strategy
#[derive(Debug, Clone)]
pub struct Bot {
    strategy: StrategyDto,
    dag: Dag,
    calcs: HashMap<TimeSeriesName, CalculationDto>,
}

impl Bot {
    pub fn new(strategy: StrategyDto) -> GenResult<Self> {
        let dag = Dag::new(strategy.clone())?;
        let calcs: HashMap<String, CalculationDto> = strategy
            .calcs()
            .iter()
            .map(|calc| (calc.name().to_string(), calc.clone()))
            .collect();
        Ok(Bot {
            strategy,
            dag,
            calcs,
        })
    }
    fn strategy(&self) -> &StrategyDto {
        &self.strategy
    }
    fn calc(&self, name: &str) -> Result<&CalculationDto, &str> {
        self.calcs.get(name).ok_or("not found")
    }
    pub fn asset_score(
        &self,
        asset: Asset,
        timestamp: TimeStamp,
        data_client: Box<dyn DataClient>,
    ) -> GenResult<AssetScore> {
        let mut exe_bot = ExecutableBot {
            asset,
            timestamp,
            execution_order: self.dag.execution_order().clone(),
            calcs: self.calcs.clone(),
            data_client,
            calc_status: self
                .calcs
                .keys()
                .map(|c| (c.clone(), CalculationStatus::NotStarted))
                .collect(),
            calc_time_series: HashMap::new(),
        };
        exe_bot.execute()?;
        Ok(AssetScore::new(exe_bot))
        // Ok(exe_bot)
    }
}

/// Composes a `Bot` with a `Asset`, `Timestamp` and `DataClient`.
#[derive(Debug)]
pub struct ExecutableBot {
    asset: Asset,
    timestamp: TimeStamp,
    execution_order: Vec<TimeSeriesName>,
    calcs: HashMap<TimeSeriesName, CalculationDto>,
    data_client: Box<dyn DataClient>,
    calc_status: HashMap<TimeSeriesName, CalculationStatus>,
    calc_time_series: HashMap<TimeSeriesName, TimeSeries1D>,
}

impl ExecutableBot {
    pub(crate) fn overall_status(&self) -> CalculationStatus {
        // compute group by count using Entry Api
        let mut count_by_status: HashMap<CalculationStatus, usize> = HashMap::new();
        for (time_series_name, calc_status) in &self.calc_status {
            let count = count_by_status.entry(calc_status.clone()).or_insert(1usize);
            *count += 1;
        }
        // declare determining factors of overall status
        let has_error = match count_by_status.get(&CalculationStatus::Error) {
            Some(_) => true,
            None => false,
        };
        let all_complete = match count_by_status.get(&CalculationStatus::Complete) {
            Some(n) => n == &self.calcs.len(),
            None => false,
        };
        let all_not_started = match count_by_status.get(&CalculationStatus::NotStarted) {
            Some(n) => n == &self.calcs.len(),
            None => false,
        };
        // apply business logic against factors
        if has_error {
            CalculationStatus::Error
        } else if all_complete {
            CalculationStatus::Complete
        } else if all_not_started {
            CalculationStatus::NotStarted
        } else {
            CalculationStatus::InProgress
        }
    }
    fn status(&mut self, calc_name: &str, new_calc_status: CalculationStatus) {
        if let Some(calc_status) = self.calc_status.get_mut(calc_name) {
            *calc_status = new_calc_status;
        }
    }

    pub fn upstream(&self, calc_name: &str) -> GenResult<&TimeSeries1D> {
        match self.calc_time_series.get(calc_name) {
            Some(time_series_) => Ok(time_series_),
            None => Err(UpstreamNotFoundError::new(calc_name.to_string())),
        }
    }

    pub fn score(&self) -> GenResult<&DataPointValue> {
        match self
            .upstream(self.execution_order.last().expect("impossible"))?
            .values()
            .last()
        {
            Some(score) => Ok(score),
            None => Err(UpstreamNotFoundError::new(format!(
                "score calc: {}",
                self.execution_order.last().expect("impossible")
            ))),
        }
    }

    /// Traverse `Dag` executing each node for given `Asset` as of `Timestamp`
    pub fn execute(&mut self) -> GenResult<()> {
        let calc_order = self.execution_order.clone();
        for calc_name in calc_order {
            println!("\nexecuting {}", calc_name);
            self.status(&calc_name, CalculationStatus::InProgress);
            let calc = self.calcs.get(&calc_name).ok_or("calc not found")?;

            let calc_time_series = match calc.operation() {
                Operation::QUERY => self.handle_query(calc),
                Operation::ADD => self.handle_add(calc),
                Operation::SUB => self.handle_sub(calc),
                Operation::MUL => self.handle_mul(calc),
                Operation::DIV => self.handle_div(calc),
                Operation::TS_ADD => self.handle_ts_add(calc),
                Operation::TS_SUB => self.handle_ts_sub(calc),
                Operation::TS_MUL => self.handle_ts_mul(calc),
                Operation::TS_DIV => self.handle_ts_div(calc),
                Operation::SMA => self.handle_sma(calc),
            };
            self.status(
                &calc_name,
                match calc_time_series.is_ok() {
                    true => CalculationStatus::Complete,
                    false => CalculationStatus::Error,
                },
            );

            self.calc_time_series
                .insert(calc_name.clone(), calc_time_series?);
        }
        Ok(())
    }
    // TODO parameterized query: generalize market data retrieval
    fn handle_query(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::QUERY);
        let query_dto: QueryCalculationDto = calculation_dto.clone().try_into()?;
        Ok(self
            .data_client
            .query(&self.asset, &self.timestamp, Some(query_dto))?
            .clone())
    }
    fn handle_add(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::ADD);
        let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
            calculation_dto.clone().try_into()?;
        let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
        Ok(time_series.add(dyadic_scalar_calc_dto.scalar()))
    }
    fn handle_sub(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::SUB);
        let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
            calculation_dto.clone().try_into()?;
        let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
        Ok(time_series.sub(dyadic_scalar_calc_dto.scalar()))
    }
    fn handle_mul(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::MUL);
        let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
            calculation_dto.clone().try_into()?;
        let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
        Ok(time_series.mul(dyadic_scalar_calc_dto.scalar()))
    }
    fn handle_div(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::DIV);
        let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
            calculation_dto.clone().try_into()?;
        let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
        Ok(time_series.div(dyadic_scalar_calc_dto.scalar()))
    }
    fn handle_ts_add(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::TS_ADD);
        let dyadic_ts_calc_dto: DyadicTsCalculationDto = calculation_dto.clone().try_into()?;
        let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
        let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
        Ok(left_value.ts_add(right_value))
    }
    fn handle_ts_sub(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::TS_SUB);
        let dyadic_ts_calc_dto: DyadicTsCalculationDto = calculation_dto.clone().try_into()?;
        let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
        let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
        Ok(left_value.ts_sub(right_value))
    }
    fn handle_ts_mul(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::TS_MUL);
        let dyadic_ts_calc_dto: DyadicTsCalculationDto = calculation_dto.clone().try_into()?;
        let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
        let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
        Ok(left_value.ts_mul(right_value))
    }
    fn handle_ts_div(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::TS_DIV);
        let dyadic_ts_calc_dto: DyadicTsCalculationDto = calculation_dto.clone().try_into()?;
        let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
        let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
        Ok(left_value.ts_div(right_value))
    }
    fn handle_sma(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::SMA);
        let sma_dto: SmaCalculationDto = calculation_dto.clone().try_into()?;
        let time_series = self.upstream(sma_dto.time_series())?;
        Ok(time_series.sma(sma_dto.window_size()))
    }
}

#[derive(Debug)]
pub struct AssetScore {
    asset: Asset,
    timestamp: TimeStamp,
    status: CalculationStatus,
    calc_status: HashMap<TimeSeriesName, CalculationStatus>,
    calc_time_series: HashMap<TimeSeriesName, TimeSeries1D>,
}

impl AssetScore {
    fn new(bot: ExecutableBot) -> AssetScore {
        let overall_status = bot.overall_status();
        AssetScore {
            asset: bot.asset,
            timestamp: bot.timestamp,
            status: overall_status,
            calc_status: bot.calc_status,
            calc_time_series: bot.calc_time_series,
        }
    }
    pub fn asset(&self) -> &Asset {
        &self.asset
    }
    pub fn timestamp(&self) -> usize {
        self.timestamp
    }
    pub fn calc_status(&self) -> &HashMap<TimeSeriesName, CalculationStatus> {
        &self.calc_status
    }
    pub fn calc_time_series(&self) -> &HashMap<TimeSeriesName, TimeSeries1D> {
        &self.calc_time_series
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bot::{AssetScore, Bot, CalculationStatus};
    use crate::data::Asset;
    use crate::errors::GenResult;
    use crate::simulation::{MockDataClient, TODAY};
    use crate::strategy::{
        from_path, CalculationDto, OperandDto, OperandType, Operation, ScoreDto, StrategyDto,
    };
    use std::collections::HashMap;

    fn strategy_fixture() -> StrategyDto {
        StrategyDto::new(
            String::from("Small Strategy Document"),
            ScoreDto::new(String::from("price")),
            vec![CalculationDto::new(
                String::from("price"),
                Operation::QUERY,
                vec![OperandDto::new(
                    String::from("field"),
                    OperandType::Text,
                    String::from("close"),
                )],
            )],
        )
    }

    fn bot_fixture() -> GenResult<Bot> {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        Bot::new(strategy)
    }

    #[test]
    fn execute() -> GenResult<()> {
        let bot = bot_fixture()?;
        let asset = Asset::new(String::from("A"));
        let timestamp = TODAY;
        let data_client = MockDataClient::new();
        let asset_score: AssetScore = bot.asset_score(asset, timestamp, Box::new(data_client))?;
        // let asset_score = AssetScore::new(executable_bot);
        // executable_bot
        //     .calc_time_series
        //     .values()
        //     .for_each(|time_series| assert!(time_series.len() > 0));
        Ok(())
    }

    #[test]
    fn group_by_test() {
        let data: HashMap<usize, i32> = vec![(1usize, -1), (10usize, -10), (100usize, -10)]
            .into_iter()
            .collect();

        println!("{:?}", data);
    }
}
