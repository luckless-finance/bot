use std::borrow::Borrow;

type DataValue = f64;

trait Data {
    fn value(&self) -> DataValue;
}

trait Dag {
    fn root(&self) -> Box<&dyn DagNode>;
    fn leaves(&self) -> Vec<&dyn DagNode>;
    fn execute(&self) -> Box<dyn Data>;
}

trait DagNode {
    fn dependencies(&self) -> Vec<&dyn DagNode>;
    fn execute(&mut self) -> Box<dyn Data>;
}

trait ScoreNode {
    fn score(&self) -> DataValue;
}

struct StaticNumberData {
    value: f64
}

impl StaticNumberData {
    pub fn new(value: f64) -> Box<Self> {
        Box::from(StaticNumberData { value })
    }
}


impl Data for StaticNumberData {
    fn value(&self) -> DataValue {
        self.value
    }
}

struct TimeSeriesData {}

impl TimeSeriesData {
    pub fn new() -> Box<Self> {
        Box::from(TimeSeriesData {})
    }
}

impl Data for TimeSeriesData {
    fn value(&self) -> DataValue {
        666.
    }
}

struct DagImpl {}

impl Dag for DagImpl {
    fn root(&self) -> Box<&dyn DagNode> {
        unimplemented!()
    }

    fn leaves(&self) -> Vec<&dyn DagNode> {
        unimplemented!()
    }

    fn execute(&self) -> Box<dyn Data> {
        unimplemented!()
    }
}

struct StaticNumberNode {
    number: f64
}

impl StaticNumberNode {
    pub fn new(number: f64) -> Box<Self> {
        Box::from(StaticNumberNode { number })
    }
}

impl DagNode for StaticNumberNode {
    fn dependencies(&self) -> Vec<&dyn DagNode> {
        vec![]
    }

    fn execute(&mut self) -> Box<dyn Data> {
        StaticNumberData::new(self.number)
    }
}

impl ScoreNode for StaticNumberNode {
    fn score(&self) -> f64 {
        self.number
    }
}

struct AdditionNode {
    lhs: Box<dyn DagNode>,
    rhs: Box<dyn DagNode>,
    result: Option<DataValue>,
}

impl AdditionNode {
    pub fn new(lhs: Box<dyn DagNode>,
               rhs: Box<dyn DagNode>) -> Box<Self> {
        Box::from(AdditionNode {
            lhs: Box::from(lhs),
            rhs: Box::from(rhs),
            result: None,
        })
    }
}

impl DagNode for AdditionNode {
    fn dependencies(&self) -> Vec<&dyn DagNode> {
        vec![]
    }

    fn execute(&mut self) -> Box<dyn Data> {
        self.result = match self.result {
            None => { Some(&self.lhs.execute().value() + &self.lhs.execute().value()) }
            _ => self.result
        };
        StaticNumberData::new(
            self.result.unwrap()
        )
    }
}

impl ScoreNode for AdditionNode {
    fn score(&self) -> f64 {
        3.0
    }
}

struct SummationNode {
    summands: Vec<Box<dyn DagNode>>
}

impl SummationNode {
    pub fn new(summands: Vec<Box<dyn DagNode>>) -> Self {
        SummationNode { summands }
    }
}

impl DagNode for SummationNode {
    fn dependencies(&self) -> Vec<&dyn DagNode> {
        self.summands.iter().map(|x| x.borrow()).collect()
    }

    fn execute(&mut self) -> Box<dyn Data> {
        StaticNumberData::new(
            self.dependencies().iter_mut()
                .map(|x| x.execute())
                .map(|x|x.borrow().value())
                .sum()
        )
    }
}

struct QueryNode {}

impl QueryNode {
    pub fn new() -> Box<Self> {
        Box::from(QueryNode {})
    }
}

impl DagNode for QueryNode {
    fn dependencies(&self) -> Vec<&dyn DagNode> {
        vec![]
    }

    fn execute(&mut self) -> Box<dyn Data> {
        TimeSeriesData::new()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::dag_flow::{AdditionNode, DagNode, QueryNode, StaticNumberNode, SummationNode};

    #[test]
    fn test_static_number_node() {
        assert_eq!(5.0,
                   StaticNumberNode::new(5.0).execute().value());
    }

    #[test]
    fn test_add_node() {
        assert_eq!(10.0,
                   AdditionNode::new(StaticNumberNode::new(5.0),
                                     StaticNumberNode::new(5.0)).execute().value());
    }

    #[test]
    fn test_sum_node() {
        assert_eq!(18.0, SummationNode::new(vec![
            StaticNumberNode::new(5.0),
            StaticNumberNode::new(6.0),
            StaticNumberNode::new(7.0)]).execute().value());
    }


    #[test]
    fn test_query_node() {
        assert_eq!(QueryNode::new().execute().value(), 666.)
    }
}