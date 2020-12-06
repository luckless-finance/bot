#![allow(dead_code)]
// #![allow(unused)]

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;

use petgraph::algo::{connected_components, is_cyclic_directed, toposort};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;

use crate::dto::StrategyDTO;

pub struct Dag {
    dag_dto: DagDTO,
    node_lkup: HashMap<String, NodeIndex>,
}

impl Dag {
    pub fn new(dag_dto: DagDTO) -> Self {
        let node_lkup: HashMap<String, NodeIndex<u32>> = dag_dto
            .node_indices()
            .into_iter()
            .map(|idx| (dag_dto.node_weight(idx).expect("impossible").clone(), idx))
            .collect();
        Dag { dag_dto, node_lkup }
    }
    pub fn upstream(&self, node: &String) -> Vec<String> {
        self.dag_dto
            .neighbors_directed(
                self.node_lkup.get(node).expect("node not found").clone(),
                Direction::Incoming,
            )
            .map(|x| self.dag_dto.node_weight(x).expect("node not found").clone())
            .collect()
    }
}

pub type DagDTO = DiGraph<String, String>;

pub fn execution_order(g: &DagDTO) -> Vec<&String> {
    toposort(g, None)
        .expect("unable to toposort")
        .iter()
        .map(|node_idx: &NodeIndex| g.node_weight(*node_idx).unwrap())
        .collect()
}

pub fn to_dot_text(g: &DagDTO) -> String {
    format!("{:?}", Dot::with_config(g, &[Config::EdgeNoLabel]))
}

pub fn to_dot_file(g: &DagDTO) {
    let mut output_file = File::create(
        current_dir()
            .expect("unable to find current_dir")
            .join("output.dot"),
    )
    .expect("unable to open output file");
    let dot_text = to_dot_text(g);
    output_file
        .write_all(dot_text.as_bytes())
        .expect("unable to write file");
}

pub fn to_dag(strategy: &StrategyDTO) -> Result<DagDTO, &str> {
    let mut dag: DagDTO = DiGraph::new();
    let mut node_lookup = HashMap::new();

    // add nodes
    for calc in strategy.calcs() {
        let index = dag.add_node(calc.name().to_string());
        node_lookup.insert(calc.name(), index);
    }
    // add edges
    for calc in strategy.calcs() {
        for op in calc.operands() {
            if node_lookup.contains_key(op.value()) {
                let operand = node_lookup.get(op.value()).expect("operand not found");
                let calc = node_lookup.get(calc.name()).expect("calc not found");
                dag.add_edge(*operand, *calc, String::new());
            }
        }
    }
    match is_cyclic_directed(&dag) {
        true => Err("cyclic"),
        false => match connected_components(&dag) {
            0 => Err("zero connected components found"),
            1 => Ok(dag),
            _ => Err("more than 1 connected component found"),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::path::Path;

    use petgraph::prelude::*;

    use crate::dag::{execution_order, to_dag, to_dot_file, Dag, DagDTO};
    use crate::dto::{from_path, StrategyDTO};

    fn strategy_fixture() -> StrategyDTO {
        from_path(Path::new("strategy.yaml")).expect("unable to load strategy")
    }

    fn dag_fixture() -> DagDTO {
        to_dag(strategy_fixture().borrow()).expect("unable to convert to bot")
    }

    #[test]
    fn strategy_to_dag() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy).expect("unable to convert to bot");
        to_dot_file(&dag);
        assert_eq!(dag.node_count(), 5);
        assert_eq!(dag.edge_count(), 6);
        let nodes = dag
            .node_indices()
            .map(|i| dag.node_weight(i).expect("node not found"))
            .find(|d| d.as_str().eq("sma200"))
            .expect("sma200 not found");
        assert_eq!(nodes, "sma200")
    }

    #[test]
    fn traverse_dag_order() {
        let dag: DagDTO = dag_fixture();
        let exe_order = execution_order(&dag);

        let node_execution_order_lkup: HashMap<&String, usize> = (0..exe_order.len())
            .into_iter()
            .map(|position| (exe_order.get(position).unwrap().clone(), position))
            .collect();
        let order_constraints = &[
            &["close", "sma50", "sma_diff", "sma_gap"],
            &["close", "sma200", "sma_diff", "sma_gap"],
        ];
        for outer_idx in 0..order_constraints.len() {
            let expected_order = order_constraints[outer_idx];
            for inner_idx in 0..(expected_order.len() - 1) {
                let a = expected_order[inner_idx];
                let a_position = node_execution_order_lkup.get(&a.to_string()).unwrap();
                let b = expected_order[inner_idx + 1];
                let b_position = node_execution_order_lkup.get(&b.to_string()).unwrap();
                // println!("{:?}", (a, b));
                // println!("{:?}", (a_position, b_position));
                assert!(a_position < b_position);
            }
        }
    }

    #[test]
    fn dag_to_dot_file() {
        let mut dag: DagDTO = DiGraph::new();
        let node_index_a: NodeIndex = dag.add_node(String::from("A"));
        let node_index_b = dag.add_node(String::from("B"));
        let node_index_c = dag.add_node(String::from("C"));
        let node_index_d = dag.add_node(String::from("D"));
        dag.add_edge(node_index_a, node_index_b, String::from(""));
        dag.add_edge(node_index_a, node_index_c, String::from(""));
        dag.add_edge(node_index_b, node_index_d, String::from(""));
        dag.add_edge(node_index_c, node_index_d, String::from(""));
        to_dot_file(&dag);
        let expected_output =
            read_to_string("expected_output.dot").expect("expected_output.dot not found.");
        let output = read_to_string("output.dot").expect("output.dot not found.");
        assert_eq!(output, expected_output);
    }

    #[test]
    fn dag_upstream() {
        let dag_dto = dag_fixture();
        let dag = Dag::new(dag_dto);
        let upstream = dag.upstream(&String::from("sma50"));
        assert_eq!(upstream, vec![String::from("close")]);

        let upstream = dag.upstream(&String::from("sma_gap"));
        assert!(upstream.contains(&String::from("sma_diff")));
        assert!(upstream.contains(&String::from("sma50")));

        println!("{:?}", upstream);
    }
}

#[cfg(test)]
mod learn_library {
    use std::path::Path;

    use petgraph::algo::toposort;
    use petgraph::prelude::*;

    use crate::dag::{to_dag, DagDTO};
    use crate::dto::from_path;

    #[test]
    fn topo() {
        // dag = C -> B <- A
        let mut dag: DagDTO = DiGraph::new();
        let b = dag.add_node(String::from("B"));
        let c = dag.add_node(String::from("C"));
        let a = dag.add_node(String::from("A"));
        dag.add_edge(a, b, String::new());
        dag.add_edge(c, b, String::new());
        let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

        let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
        let node = dag.node_weight(*leaf_node_idx).unwrap();
        // println!("{:?}", node);
        assert_eq!(node, "A");

        let leaf_node_idx = sorted_node_ids.get(1).expect("unable to get leaf");
        let node = dag.node_weight(*leaf_node_idx).unwrap();
        // println!("{:?}", node);
        assert_eq!(node, "C");
    }

    #[test]
    fn dfs_post_order() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag: DagDTO = to_dag(&strategy).expect("unable to convert to bot");
        let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

        let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
        let leaf_node = dag
            .node_weight(*leaf_node_idx)
            .expect("unable to find node");
        assert_eq!(leaf_node, "close");

        let mut dfs_post_order = DfsPostOrder::new(&dag, *leaf_node_idx);
        let root_node_id = dfs_post_order.next(&dag).unwrap();
        let root_node: &String = dag.node_weight(root_node_id).expect("unable to find root");
        // println!("{:?}", root_node);
        assert_eq!(root_node, strategy.score().calc());
    }

    #[test]
    fn bfs() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag: DagDTO = to_dag(&strategy).expect("unable to convert to bot");
        let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

        let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
        let leaf_node = dag
            .node_weight(*leaf_node_idx)
            .expect("unable to find node");
        assert_eq!(leaf_node, "close");

        let mut bfs = Bfs::new(&dag, *leaf_node_idx);

        let mut node: &String;
        loop {
            let node_id = bfs.next(&dag).unwrap();
            node = dag.node_weight(node_id).expect("unable to find root");
            // println!("{:?}", node);
            if node == strategy.score().calc() {
                break;
            }
        }
        assert_eq!(node, strategy.score().calc());
    }
}
