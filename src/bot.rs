#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, DataClient};
use crate::dto::{CalculationDTO, Operation, StrategyDTO};
use crate::time_series::{TimeSeries1D, TimeStamp};

/// Wraps several DTOs required traverse and consume a strategy
#[derive(Debug, Clone)]
pub(crate) struct Bot {
    strategy: StrategyDTO,
    dag: Dag,
    calc_lkup: HashMap<String, CalculationDTO>,
}

/// Composes a `Bot` with a `Asset`, `Timestamp` and `DataClient`.
pub(crate) struct ExecutableBot {
    strategy: StrategyDTO,
    dag: Dag,
    calc_lkup: HashMap<String, CalculationDTO>,
    asset: Asset,
    timestamp: TimeStamp,
    data_client: Box<dyn DataClient>,
    calc_status_lkup: HashMap<String, CalculationStatus>,
    calc_data_lkup: HashMap<String, TimeSeries1D>,
}

// TODO implement handlers and result memoization
impl ExecutableBot {
    fn set_status(&mut self, calc_name: &str, new_calc_status: CalculationStatus) {
        if let Some(calc_status) = self.calc_status_lkup.get_mut(calc_name) {
            *calc_status = new_calc_status;
        }
    }

    /// Traverse `Dag` executing each node for given `Asset` as of `Timestamp`
    fn execute(&mut self) -> Result<(), String> {
        let calc_order = self.dag.execution_order().clone();
        for calc_name in calc_order {
            println!("\nexecuting {}", calc_name);
            self.set_status(&calc_name, CalculationStatus::InProgress);
            let calc = self.calc_lkup.get(&calc_name).expect("calc not found");

            // TODO add ADD, MUL
            // TODO align semantics with TimeSeries1D
            let calc_time_series = match calc.operation() {
                Operation::DIV => self.handle_div(calc),
                Operation::SMA => self.handle_sma(calc),
                Operation::SUB => self.handle_sub(calc),
                Operation::QUERY => self.handle_query(calc),
            };
            self.set_status(
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

    // TODO enforce DTO constraints at parse time
    fn handle_div(&self, calc: &CalculationDTO) -> Result<TimeSeries1D, String> {
        assert_eq!(*calc.operation(), Operation::DIV);
        // TODO replace magic strings
        assert_eq!(
            calc.operands().len(),
            2,
            "DIV operation requires operands: 'numerator' and 'denominator'"
        );
        let numerator_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'numerator'
            .find(|o| o.name() == "numerator")
            .unwrap();
        let denominator_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'denominator'
            .find(|o| o.name() == "denominator")
            .unwrap();
        let numerator_value = self.calc_data_lkup.get(numerator_operand.value()).unwrap();
        let denominator_value = self
            .calc_data_lkup
            .get(denominator_operand.value())
            .unwrap();
        Ok(numerator_value.div(denominator_value.clone()))
    }

    // TODO enforce DTO constraints at parse time
    fn handle_sma(&self, calc: &CalculationDTO) -> Result<TimeSeries1D, String> {
        assert_eq!(*calc.operation(), Operation::SMA);
        assert_eq!(
            calc.operands().len(),
            2,
            "SMA operation requires operands: 'window_size' and 'time_series'"
        );
        let window_size_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'window_size'
            .find(|o| o.name() == "window_size")
            .expect("SMA operation requires operand: 'window_size'");
        let time_series_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'time_series'
            .find(|o| o.name() == "time_series")
            .expect("SMA operation requires operand: 'time_series'");
        let time_series_value = self
            .calc_data_lkup
            .get(time_series_operand.value())
            .expect("Upstream TimeSeries1D not found.");
        let window_size_value: usize = window_size_operand
            .value()
            .parse()
            .expect("'window_size' must be usize");
        Ok(time_series_value.sma(window_size_value))
    }

    // TODO enforce DTO constraints at parse time
    fn handle_sub(&self, calc: &CalculationDTO) -> Result<TimeSeries1D, String> {
        assert_eq!(*calc.operation(), Operation::SUB);
        // TODO replace magic strings
        assert_eq!(
            calc.operands().len(),
            2,
            "SUB operation requires operands: 'left' and 'right'"
        );
        let left_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'left'
            .find(|o| o.name() == "left")
            .unwrap();
        let right_operand = calc
            .operands()
            .iter()
            // TODO replace magic strings 'right'
            .find(|o| o.name() == "right")
            .unwrap();
        let left_value = self.calc_data_lkup.get(left_operand.value()).unwrap();
        let right_value = self.calc_data_lkup.get(right_operand.value()).unwrap();
        // TODO wasteful clone, sub calls aligns which clones
        Ok(left_value.sub(right_value.clone()))
    }

    // TODO parameterized query: generalize market data retrieval
    fn handle_query(&self, calc: &CalculationDTO) -> Result<TimeSeries1D, String> {
        assert_eq!(*calc.operation(), Operation::QUERY);
        println!("TODO execute {}", calc.name());
        let name = "field";
        let _field: &str = calc
            .operands()
            .iter()
            .find(|o| o.name() == name)
            .expect("symbol operand not found")
            .value();
        self.data_client.query(&self.asset, &self.timestamp)
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
    pub fn new(strategy: StrategyDTO) -> Box<Self> {
        let dag = Dag::new(strategy.clone());
        let calc_lkup: HashMap<String, CalculationDTO> = strategy
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
    fn strategy(&self) -> &StrategyDTO {
        &self.strategy
    }
    pub fn calc(&self, name: &str) -> Result<&CalculationDTO, &str> {
        self.calc_lkup.get(name).ok_or("not found")
    }
    pub fn queries(&self) -> Vec<&CalculationDTO> {
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
        from_path, CalculationDTO, OperandDTO, OperandType, Operation, ScoreDTO, StrategyDTO,
    };

    fn strategy_fixture() -> StrategyDTO {
        StrategyDTO::new(
            String::from("Small Strategy Document"),
            ScoreDTO::new(String::from("price")),
            vec![CalculationDTO::new(
                String::from("price"),
                Operation::QUERY,
                vec![OperandDTO::new(
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
