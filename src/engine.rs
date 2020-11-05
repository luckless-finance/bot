use petgraph::prelude::DiGraph;

use crate::dag::Dag;
use crate::dto::{Calculation, Strategy};

pub struct Bot {
    strategy: Strategy,
    dag: Dag,
}

impl Bot {
    pub fn new(strategy: Strategy, dag: Dag) -> Self {
        Bot { strategy, dag }
    }
    pub fn dag(&self) -> &Dag {
        &self.dag
    }
    pub fn strategy(&self) -> &Strategy {
        &self.strategy
    }
    pub fn queries(&self) -> Vec<&Calculation> {
        self.strategy.calculations().iter()
            .filter(|c| (c.operation()) == "query")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::dag::to_dag;
    use crate::dto::{Calculation, from_path};
    use crate::engine::Bot;

    #[test]
    fn queries_test() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy).expect("unable to convert to dag");
        let bot = Bot::new(strategy, dag);
        let close_queries = bot.queries().iter()
            .filter(|calculation| (**calculation).name() == "close")
            .count();
        assert_eq!(close_queries, 1);
    }
}