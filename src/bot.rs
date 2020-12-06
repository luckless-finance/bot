#![allow(dead_code)]

use std::collections::HashMap;

use petgraph::prelude::*;

use crate::dag::{DagDTO, execution_order, to_dag};
use crate::data::{Asset, MockDataClient};
use crate::dto::{CalculationDTO, StrategyDTO};
use crate::time_series::{TimeSeries1D, TimeStamp};

pub struct Bot {
    strategy: StrategyDTO,
    dag: DagDTO,
    calculation_lkup: HashMap<String, CalculationDTO>,
}

pub struct ExecutableBot {
    // TODO replace type with trait DataClient
    data_client: MockDataClient,
    calculation_status_lkup: HashMap<String, CalculationStatus>,
}

pub enum CalculationStatus {
    NotStarted,
    Complete,
}

impl Bot {
    pub fn new(strategy: StrategyDTO) -> Box<Self> {
        let dag = to_dag(&strategy).expect("unable to build bot");
        let calc_lkup: HashMap<&str, &CalculationDTO> = strategy.calculations().iter().map(|calc| (calc.name().to_string(), calc)).collect();

        let node_statuses: HashMap<&String, CalculationStatus> = bot
            .dag
            .node_indices()
            .into_iter()
            .map(|node_idx: NodeIndex| {
                (
                    bot.dag.node_weight(node_idx).unwrap(),
                    CalculationStatus::NotStarted,
                )
            })
            .collect();
        // let data_client = MockDataClient::new();
        Box::from(Bot {
            strategy,
            dag,
            // data_client,
        })
    }
    fn dag(&self) -> &DagDTO {
        &self.dag
    }
    fn strategy(&self) -> &StrategyDTO {
        &self.strategy
    }
    pub fn queries(&self) -> Vec<&CalculationDTO> {
        self.strategy
            .calculations()
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
    //     let _score_calc = self.strategy.score().calculation();
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;

    use petgraph::algo::toposort;
    use petgraph::graph::NodeIndex;

    use crate::bot::{Bot, CalculationStatus};
    use crate::dag::execution_order;
    use crate::data::TODAY;
    use crate::dto::from_path;
    use crate::time_series::TimeSeries1D;

    fn bot_fixture() -> Box<Bot> {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        Bot::new(strategy)
    }

    #[test]
    fn queries() {
        let bot = bot_fixture();
        let close_queries = bot
            .queries()
            .iter()
            .filter(|calculation| (**calculation).name() == "close")
            .count();
        assert_eq!(close_queries, 1);
    }

    #[test]
    fn execute() {
        let bot: Box<Bot> = bot_fixture();
        let execution_order: Vec<&String> = execution_order(bot.dag());
        let node_data: HashMap<String, TimeSeries1D> = HashMap::new();
        let node_statuses: HashMap<&String, CalculationStatus> = bot
            .dag
            .node_indices()
            .into_iter()
            .map(|node_idx: NodeIndex| {
                (
                    bot.dag.node_weight(node_idx).unwrap(),
                    CalculationStatus::NotStarted,
                )
            })
            .collect();

        // execute leaves (queries)
        for node_idx in toposort(&bot.dag, None).expect("unable to toposort").iter() {
            println!("{:?}", bot.dag.node_weight(*node_idx));
        }
        ()
    }
}
