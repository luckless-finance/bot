#![allow(dead_code)]

use crate::dag::{to_dag, Dag};
use crate::data::{Asset, MockDataClient};
use crate::strategy::{Calculation, StrategyDTO};
use crate::time_series::{DataPointValue, TimeStamp};

pub struct Bot {
    strategy: StrategyDTO,
    dag: Dag,
    data_client: MockDataClient,
}

trait ExecutableBot {
    fn execute(&self, asset: &Asset, timestamp: &TimeStamp) -> DataPointValue;
}

impl Bot {
    pub fn new(strategy: StrategyDTO) -> Box<Self> {
        let dag = to_dag(&strategy).expect("unable to build bot");
        let data_client = MockDataClient::new();
        Box::from(Bot {
            strategy,
            dag,
            data_client,
        })
    }
    fn dag(&self) -> &Dag {
        &self.dag
    }
    fn strategy(&self) -> &StrategyDTO {
        &self.strategy
    }
    pub fn queries(&self) -> Vec<&Calculation> {
        self.strategy
            .calculations()
            .iter()
            .filter(|c| (c.operation()) == "query")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;

    use crate::bot::Bot;
    use crate::data::TODAY;
    use crate::strategy::from_path;
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
    fn bot() {
        let bot = bot_fixture();
        let symbol = "B";
        let _dag_node_output_lookup: HashMap<String, TimeSeries1D> = bot
            .queries()
            .iter()
            .map(|c| {
                (
                    c.name().to_string(),
                    bot.data_client.query(
                        bot.data_client
                            .assets()
                            .get(symbol)
                            .expect(&*format!("Query Failed. Asset not found for: {:?}", c)),
                        &TODAY,
                    ),
                )
            })
            .collect();
    }
}
