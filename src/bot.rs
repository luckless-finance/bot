#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::Dag;
use crate::data::{Asset, MockDataClient};
use crate::dto::{CalculationDTO, StrategyDTO};
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

impl ExecutableBot {
    fn execute(&self) {
        let calc_order = &self.bot.dag.execution_order();
        for calc in calc_order {
            println!("\nexecuting {}", calc);
        }
    }
}

#[derive(Debug, Clone)]
pub enum CalculationStatus {
    NotStarted,
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
            .filter(|c| (c.operation()) == "query")
            .collect()
    }

    pub fn as_executable(&self, asset: Asset, timestamp: TimeStamp) -> ExecutableBot {
        ExecutableBot {
            asset,
            timestamp,
            bot: self.clone(),
            data_client: MockDataClient::new(),
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
    use crate::data::{Asset, TODAY};
    use crate::dto::from_path;

    fn bot_fixture() -> Box<Bot> {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        Bot::new(strategy)
    }

    #[test]
    fn execute() {
        let bot = bot_fixture();
        let asset = Asset::new(String::from("A"));
        let timestamp = TODAY;
        let executable_bot = bot.as_executable(asset, timestamp);
        executable_bot.execute();
    }

    #[test]
    fn queries() {
        let bot = bot_fixture();
        let close_queries = bot
            .queries()
            .iter()
            .filter(|calc| (**calc).name() == "close")
            .count();
        assert_eq!(close_queries, 1);
    }
}
