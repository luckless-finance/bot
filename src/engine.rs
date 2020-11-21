use crate::dag::{Dag, to_dag};
use crate::data::{DataPointValue, TS};
use crate::dto::{Calculation, Strategy};

pub struct Bot {
    strategy: Strategy,
    dag: Dag,
}

impl Bot {
    pub fn new(strategy: Strategy) -> Self {
        Bot { strategy, dag: to_dag(&strategy).expect("unable to build dag") }
    }
    fn dag(&self) -> &Dag {
        &self.dag
    }
    fn strategy(&self) -> &Strategy {
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

trait BackTest {

}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::dag::to_dag;
    use crate::dto::from_path;
    use crate::engine::Bot;

    #[test]
    fn queries_test() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let bot = Bot::new(strategy);
        let close_queries = bot
            .queries()
            .iter()
            .filter(|calculation| (**calculation).name() == "close")
            .count();
        assert_eq!(close_queries, 1);
    }

    #[test]
    fn execute_test() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let bot = Bot::new(strategy);
    }
}
