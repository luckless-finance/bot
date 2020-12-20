#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, MockDataClient, TODAY};
use crate::dto::{CalculationDTO, Operation, StrategyDTO};
use crate::time_series::{TimeSeries1D, TimeStamp};

/// Wraps several DTOs required traverse and consume a strategy
#[derive(Debug, Clone)]
pub struct Bot {
    strategy: StrategyDTO,
    dag: Dag,
    calc_lkup: HashMap<String, CalculationDTO>,
}

/// Composes a `Bot` with a `Asset`, `Timestamp` and `DataClient`.
#[derive(Debug)]
pub struct ExecutableBot {
    strategy: StrategyDTO,
    dag: Dag,
    calc_lkup: HashMap<String, CalculationDTO>,
    asset: Asset,
    timestamp: TimeStamp,
    // TODO replace type with trait DataClient
    data_client: MockDataClient,
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
        let calc_order = &self.dag.execution_order();
        for calc_name in calc_order {
            // println!("\nexecuting {}", calc_name);
            self.set_status(calc_name, CalculationStatus::InProgress);
            let calc = self.calc_lkup.get(calc_name).unwrap();
            // println!("{:?}", calc?.operation());

            let calc_time_series = match calc.operation() {
                Operation::DIV => foo_handle_div(
                    calc,
                    &self.calc_data_lkup,
                    &self.data_client,
                    &self.asset,
                    &self.timestamp,
                ),
                Operation::SMA => foo_handle_sma(
                    calc,
                    &self.calc_data_lkup,
                    &self.data_client,
                    &self.asset,
                    &self.timestamp,
                ),
                Operation::SUB => foo_handle_sub(
                    calc,
                    &self.calc_data_lkup,
                    &self.data_client,
                    &self.asset,
                    &self.timestamp,
                ),
                Operation::QUERY => foo_handle_query(
                    calc,
                    &self.calc_data_lkup,
                    &self.data_client,
                    &self.asset,
                    &self.timestamp,
                ),
            };

            if calc_time_series.is_err() {
                self.set_status(calc_name, CalculationStatus::Error);
            };

            self.calc_data_lkup
                .insert(calc_name.clone(), calc_time_series?);
        }
        Ok(())
    }
    // fn handle_div(&mut self, calc: &CalculationDTO) -> Result<TimeSeries1D, &str> {
    //     assert_eq!(*calc.operation(), Operation::DIV);
    //     println!("TODO execute {}", calc.name());
    //     let values = vec![5., 10., 15.];
    //     let index = vec![1, 3, 4];
    //     Ok(TimeSeries1D::new(index, values))
    // }
    // fn handle_sma(&mut self, calc: &CalculationDTO) -> Result<TimeSeries1D, &str> {
    //     assert_eq!(*calc.operation(), Operation::SMA);
    //     println!("TODO execute {}", calc.name());
    //     let values = vec![5., 10., 15.];
    //     let index = vec![1, 3, 4];
    //     Ok(TimeSeries1D::new(index, values))
    // }
    // fn handle_sub(&mut self, calc: &CalculationDTO) -> Result<TimeSeries1D, &str> {
    //     assert_eq!(*calc.operation(), Operation::SUB);
    //     println!("TODO execute {}", calc.name());
    //     let values = vec![5., 10., 15.];
    //     let index = vec![1, 3, 4];
    //     Ok(TimeSeries1D::new(index, values))
    // }
    // fn handle_query(&mut self, calc: &CalculationDTO) -> Result<TimeSeries1D, &str> {
    //     assert_eq!(*calc.operation(), Operation::QUERY);
    //     println!("TODO execute {}", calc.name());
    //     let name = "field";
    //     // TODO parameterized query
    //     let _field: &str = calc
    //         .operands()
    //         .iter()
    //         .find(|o| o.name() == name)
    //         .expect("symbol operand not found")
    //         .value();
    //     self.data_client.query(&self.asset, &TODAY)
    // }
}

fn foo_handle_div(
    calc: &CalculationDTO,
    calc_data_lookup: &HashMap<String, TimeSeries1D>,
    data_client: &MockDataClient,
    asset: &Asset,
    timestamp: &TimeStamp,
) -> Result<TimeSeries1D, String> {
    assert_eq!(*calc.operation(), Operation::DIV);
    println!("TODO execute {}", calc.name());
    let values = vec![5., 10., 15.];
    let index = vec![1, 3, 4];
    Ok(TimeSeries1D::new(index, values))
}

fn foo_handle_sma(
    calc: &CalculationDTO,
    calc_data_lookup: &HashMap<String, TimeSeries1D>,
    data_client: &MockDataClient,
    asset: &Asset,
    timestamp: &TimeStamp,
) -> Result<TimeSeries1D, String> {
    assert_eq!(*calc.operation(), Operation::SMA);
    println!("TODO execute {}", calc.name());
    let values = vec![5., 10., 15.];
    let index = vec![1, 3, 4];
    Ok(TimeSeries1D::new(index, values))
}

fn foo_handle_sub(
    calc: &CalculationDTO,
    calc_data_lookup: &HashMap<String, TimeSeries1D>,
    data_client: &MockDataClient,
    asset: &Asset,
    timestamp: &TimeStamp,
) -> Result<TimeSeries1D, String> {
    assert_eq!(*calc.operation(), Operation::SUB);
    println!("TODO execute {}", calc.name());
    let values = vec![5., 10., 15.];
    let index = vec![1, 3, 4];
    Ok(TimeSeries1D::new(index, values))
}

fn foo_handle_query(
    calc: &CalculationDTO,
    calc_data_lookup: &HashMap<String, TimeSeries1D>,
    data_client: &MockDataClient,
    asset: &Asset,
    timestamp: &TimeStamp,
) -> Result<TimeSeries1D, String> {
    assert_eq!(*calc.operation(), Operation::QUERY);
    println!("TODO execute {}", calc.name());
    let name = "field";
    // TODO parameterized query
    let _field: &str = calc
        .operands()
        .iter()
        .find(|o| o.name() == name)
        .expect("symbol operand not found")
        .value();
    data_client.query(&asset, &timestamp)
}

#[derive(Debug, Clone)]
pub enum CalculationStatus {
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
        data_client: MockDataClient,
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

        // let calc_order = execution_order(self.dag());

        //
        // let _dag_node_output_lookup: HashMap<String, TimeSeries1D> = HashMap::new();
        //
        // let _dag_node_output_lookup: HashMap<String, TimeSeries1D> = self
        //     .queries()
        //     .iter()
        //     .map(|c| {
        //         (
        //             c.name().to_string(),
        //             self.data_client.query(asset, timestamp),
        //         )
        //     })
        //     .collect();
        // let _score_calc = self.strategy.score().calc();
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
        let mut executable_bot = bot.as_executable(asset, timestamp, data_client);
        executable_bot.execute();
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
