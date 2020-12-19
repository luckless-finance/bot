#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, MockDataClient, TODAY};
use crate::dto::{CalculationDTO, Operation, StrategyDTO};
use crate::time_series::{TimeSeries1D, TimeStamp};

#[derive(Debug, Clone)]
pub struct Bot {
    strategy: StrategyDTO,
    dag: Dag,
    calc_lkup: HashMap<String, CalculationDTO>,
}

#[derive(Debug)]
pub struct ExecutableBot {
    bot: Bot,
    asset: Asset,
    timestamp: TimeStamp,
    // TODO replace type with trait DataClient
    data_client: MockDataClient,
    calc_status_lkup: HashMap<String, CalculationStatus>,
    calc_data_lkup: HashMap<String, TimeSeries1D>,
}

// TODO implement handlers and result memoization
impl ExecutableBot {
    fn execute(&mut self) {
        let calc_order = &self.bot.dag.execution_order();
        for calc_name in calc_order {
            println!("\nexecuting {}", calc_name);
            if let Some(calc_status) = self.calc_status_lkup.get_mut(calc_name) {
                *calc_status = CalculationStatus::InProgress;
            }
            let calc = self.bot.calc_lkup.get(calc_name).expect("calc not found");
            println!("{:?}", calc.operation());
            self.calc_data_lkup.insert(
                calc_name.clone(),
                match calc.operation() {
                    Operation::DIV => self.handle_div(calc),
                    Operation::SMA => self.handle_sma(calc),
                    Operation::SUB => self.handle_sub(calc),
                    Operation::QUERY => self.handle_query(calc),
                },
            );
        }
    }
    fn handle_div(&self, calc: &CalculationDTO) -> TimeSeries1D {
        assert_eq!(*calc.operation(), Operation::DIV);
        println!("TODO execute {}", calc.name());
        let values = vec![5., 10., 15.];
        let index = vec![1, 3, 4];
        TimeSeries1D::new(index, values)
    }
    fn handle_sma(&self, calc: &CalculationDTO) -> TimeSeries1D {
        assert_eq!(*calc.operation(), Operation::SMA);
        println!("TODO execute {}", calc.name());
        let values = vec![5., 10., 15.];
        let index = vec![1, 3, 4];
        TimeSeries1D::new(index, values)
    }
    fn handle_sub(&self, calc: &CalculationDTO) -> TimeSeries1D {
        assert_eq!(*calc.operation(), Operation::SUB);
        println!("TODO execute {}", calc.name());
        let values = vec![5., 10., 15.];
        let index = vec![1, 3, 4];
        TimeSeries1D::new(index, values)
    }
    fn handle_query(&self, calc: &CalculationDTO) -> TimeSeries1D {
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
        self.data_client.query(&self.asset, &TODAY)
    }
}

#[derive(Debug, Clone)]
pub enum CalculationStatus {
    NotStarted,
    InProgress,
    Complete,
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

    pub fn as_executable(&self, asset: Asset, timestamp: TimeStamp, data_client: MockDataClient) -> ExecutableBot {
        ExecutableBot {
            asset,
            timestamp,
            bot: self.clone(),
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
    use crate::data::{Asset, TODAY, MockDataClient};
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
