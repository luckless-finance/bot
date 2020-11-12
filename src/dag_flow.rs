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
    fn execute(&self) -> Box<dyn Data>;
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

    fn execute(&self) -> Box<dyn Data> {
        StaticNumberData::new(self.number)
    }
}

struct AdditionNode {
    lhs: Box<dyn DagNode>,
    rhs: Box<dyn DagNode>,
}

impl AdditionNode {
    pub fn new(lhs: Box<dyn DagNode>,
               rhs: Box<dyn DagNode>) -> Box<Self> {
        Box::from(AdditionNode {
            lhs: Box::from(lhs),
            rhs: Box::from(rhs),
        })
    }
}

impl DagNode for AdditionNode {
    fn dependencies(&self) -> Vec<&dyn DagNode> {
        vec![]
    }

    fn execute(&self) -> Box<dyn Data> {
        StaticNumberData::new(
            &self.lhs.execute().value() +
                &self.lhs.execute().value())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::dag_flow::{AdditionNode, DagNode, StaticNumberNode};

    #[test]
    fn test_static_number_node() {
        assert_eq!(5.0, StaticNumberNode::new(5.0).execute().value());
    }
    // #[test]
    // fn test_static_number_node() {
    //     assert_eq!(5.0, AdditionNode::new(StaticNumberNode::new(5.0), ) .execute().value());
    // }
}