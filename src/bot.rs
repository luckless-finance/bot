#![allow(dead_code)]

use std::collections::HashMap;

use crate::dag::{DagDTO, to_dag};
use crate::data::MockDataClient;
use crate::dto::{CalculationDTO, StrategyDTO};

pub struct Bot {
    strategy: StrategyDTO,
    dag: DagDTO,
    calc_lkup: HashMap<String, CalculationDTO>,
}

pub struct ExecutableBot {
    // TODO replace type with trait DataClient
    data_client: MockDataClient,
    calc_status_lkup: HashMap<String, CalculationStatus>,
}

pub enum CalculationStatus {
    NotStarted,
    Complete,
}

impl Bot {
    pub fn new(strategy: StrategyDTO) -> Box<Self> {
        let dag = to_dag(&strategy).expect("unable to build bot");
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
    fn dag(&self) -> &DagDTO {
        &self.dag
    }
    fn strategy(&self) -> &StrategyDTO {
        &self.strategy
    }
    pub fn calc(&self, name: &str) -> Result<&CalculationDTO, &str> {
        let _calc = self
            .strategy
            .calcs()
            .iter()
            .find(|calc| calc.name() == name);
        _calc.ok_or("not found")
    }
    pub fn queries(&self) -> Vec<&CalculationDTO> {
        self.strategy
            .calcs()
            .iter()
            .filter(|c| (c.operation()) == "query")
            .collect()
    }
    //
    // pub fn execute(&self, asset: &Asset, timestamp: &TimeStamp) -> () {
    //     let _dag_node_output_lookup: HashMap<String, TimeSeries1D> = HashMap::new();
    //
    //     let _dag_node_output_lookup: HashMap<String, TimeSeries1D> = self
    //         .queries()
    //         .iter()
    //         .map(|c| {
    //             (
    //                 c.name().to_string(),
    //                 self.data_client.query(asset, timestamp),
    //             )
    //         })
    //         .collect();
    //     let _score_calc = self.strategy.score().calc();
    // }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bot::Bot;
    use crate::dag::execution_order;
    use crate::dto::from_path;

    fn bot_fixture() -> Box<Bot> {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        Bot::new(strategy)
    }

    #[test]
    fn execute() {
        let bot = bot_fixture();
        let calc_order = execution_order(bot.dag());


        ()
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
