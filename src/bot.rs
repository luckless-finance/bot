#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, DataClient};
use crate::dto::{
    CalculationDto, DyadicScalarCalculationDto, DyadicTsCalculationDto, GenResult, Operation,
    QueryCalculationDto, SmaCalculationDto, StrategyDto, TimeSeriesName,
};
use crate::time_series::{TimeSeries1D, TimeStamp};
use serde::export::Formatter;
use std;
use std::convert::TryInto;
use std::fmt;

/// Wraps several Dtos required traverse and consume a strategy
#[derive(Debug, Clone)]
pub(crate) struct Bot {
    strategy: StrategyDto,
    dag: Dag,
    calc_lkup: HashMap<TimeSeriesName, CalculationDto>,
}

#[derive(Debug, Clone)]
pub(crate) struct UpstreamNotFoundError {
    upstream_name: String,
    calculation_name: String,
}

impl fmt::Display for UpstreamNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "upstream time series not found {} for calculation: {}\n",
            self.calculation_name, self.upstream_name
        )
    }
}

impl std::error::Error for UpstreamNotFoundError {
    fn description(&self) -> &str {
        "Upstream time series not found."
    }
}

/// Composes a `Bot` with a `Asset`, `Timestamp` and `DataClient`.
pub(crate) struct ExecutableBot {
    strategy: StrategyDto,
    dag: Dag,
    calc_lkup: HashMap<TimeSeriesName, CalculationDto>,
    asset: Asset,
    timestamp: TimeStamp,
    data_client: Box<dyn DataClient>,
    calc_status_lkup: HashMap<TimeSeriesName, CalculationStatus>,
    calc_data_lkup: HashMap<TimeSeriesName, TimeSeries1D>,
}

// TODO implement handlers and result memoization
impl ExecutableBot {
    fn status(&mut self, calc_name: &str, new_calc_status: CalculationStatus) {
        if let Some(calc_status) = self.calc_status_lkup.get_mut(calc_name) {
            *calc_status = new_calc_status;
        }
    }

    fn upstream(&self, calc_name: &str) -> GenResult<&TimeSeries1D> {
        match self.calc_data_lkup.get(calc_name) {
            Some(time_series_) => Ok(time_series_),
            None => Err(Box::new(UpstreamNotFoundError {
                upstream_name: "".to_string(),
                calculation_name: "".to_string(),
            })),
        }
    }

    /// Traverse `Dag` executing each node for given `Asset` as of `Timestamp`
    fn execute(&mut self) -> GenResult<()> {
        let calc_order = self.dag.execution_order().clone();
        for calc_name in calc_order {
            println!("\nexecuting {}", calc_name);
            self.status(&calc_name, CalculationStatus::InProgress);
            let calc = self.calc_lkup.get(&calc_name).ok_or("calc not found")?;

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

            self.calc_data_lkup
                .insert(calc_name.clone(), calc_time_series?);
        }
        Ok(())
    }
    // TODO parameterized query: generalize market data retrieval
    fn handle_query(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
        assert_eq!(*calculation_dto.operation(), Operation::QUERY);
        let query_dto: QueryCalculationDto = calculation_dto.clone().try_into()?;
        self.data_client
            .query(&self.asset, &self.timestamp, Some(query_dto))
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

#[derive(Debug, Clone)]
pub(crate) enum CalculationStatus {
    NotStarted,
    InProgress,
    Complete,
    Error,
}

impl Bot {
    pub fn new(strategy: StrategyDto) -> Box<Self> {
        let dag = Dag::new(strategy.clone());
        let calc_lkup: HashMap<String, CalculationDto> = strategy
            .calcs()
            .iter()
            .map(|calc| (calc.name().to_string(), calc.clone()))
            .collect();
        Box::from(Bot {
            strategy,
            dag,
            calc_lkup,
        })
    }
    fn dag(&self) -> &Dag {
        &self.dag
    }
    fn strategy(&self) -> &StrategyDto {
        &self.strategy
    }
    pub fn calc(&self, name: &str) -> Result<&CalculationDto, &str> {
        self.calc_lkup.get(name).ok_or("not found")
    }
    pub fn queries(&self) -> Vec<&CalculationDto> {
        self.strategy
            .calcs()
            .iter()
            .filter(|c| (c.operation()) == &Operation::QUERY)
            .collect()
    }

    pub fn as_executable(
        &self,
        asset: Asset,
        timestamp: TimeStamp,
        data_client: Box<dyn DataClient>,
    ) -> ExecutableBot {
        ExecutableBot {
            strategy: self.strategy.clone(),
            dag: self.dag.clone(),
            calc_lkup: self.calc_lkup.clone(),
            asset,
            timestamp,
            data_client,
            calc_status_lkup: self
                .strategy
                .calcs()
                .iter()
                .map(|c| (c.name().to_string(), CalculationStatus::NotStarted))
                .collect(),
            calc_data_lkup: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bot::Bot;
    use crate::data::{Asset, MockDataClient, TODAY};
    use crate::dto::{
        from_path, CalculationDto, OperandDto, OperandType, Operation, ScoreDto, StrategyDto,
    };

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

    fn bot_fixture() -> Box<Bot> {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        Bot::new(strategy)
    }

    #[test]
    fn execute() {
        let bot = bot_fixture();
        let asset = Asset::new(String::from("A"));
        let timestamp = TODAY;
        let data_client = MockDataClient::new();
        let mut executable_bot = bot.as_executable(asset, timestamp, Box::new(data_client));
        executable_bot.execute().expect("unable to execute");
        executable_bot
            .calc_data_lkup
            .values()
            .for_each(|time_series| assert!(time_series.len() > 0));
    }

    #[test]
    fn queries() {
        let bot = bot_fixture();
        let close_queries = bot
            .queries()
            .iter()
            .filter(|calc| (**calc).name() == "price")
            .count();
        assert_eq!(close_queries, 1);
    }
}
