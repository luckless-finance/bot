#![allow(dead_code)]

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;

use petgraph::algo::{connected_components, is_cyclic_directed};
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;

use crate::dto::StrategyDTO;

pub type DagDTO = DiGraph<String, String>;

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
    for calc in strategy.calculations() {
        let index = dag.add_node(calc.name().to_string());
        node_lookup.insert(calc.name(), index);
    }
    // add edges
    for calc in strategy.calculations() {
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
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::path::Path;

    use petgraph::algo::{is_cyclic_directed, toposort};
    use petgraph::prelude::*;
    use petgraph::visit::{Bfs, DfsPostOrder};

    use crate::dag::{to_dag, to_dot_file, DagDTO};
    use crate::dto::{from_path, StrategyDTO};

    type Dag = DiGraph<String, String>;

    #[test]
    fn traverse_dag_order() {
        let strategy: StrategyDTO =
            from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag: DagDTO = to_dag(&strategy).expect("unable to convert to bot");
        // topological sort is dag order !
        let sorted_node_ids: Vec<NodeIndex> = toposort(&dag, None).expect("unable to toposort");
        let mut node_order: HashMap<String, usize> = HashMap::new();
        let mut position = 0usize;
        for sorted_node_id in sorted_node_ids {
            println!("{:?}", dag.node_weight(sorted_node_id));
            node_order.insert(dag.node_weight(sorted_node_id).unwrap().clone(), position);
            position += 1;
        }
        let order_constraints = &[
            &["close", "sma50", "sma_diff", "sma_gap"],
            &["close", "sma200", "sma_diff", "sma_gap"],
        ];
        for outer_idx in 0..order_constraints.len() {
            let expected_order = order_constraints[outer_idx];
            for inner_idx in 0..(expected_order.len() - 1) {
                let a = expected_order[inner_idx];
                let a_position = node_order.get(a).unwrap();
                let b = expected_order[inner_idx + 1];
                let b_position = node_order.get(b).unwrap();
                println!("{:?}", (a, b));
                println!("{:?}", (a_position, b_position));
                assert!(a_position < b_position);
            }
        }
    }

    #[test]
    fn get_queries() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy).expect("unable to convert to bot");
        to_dot_file(&dag);
        println!("is a dag? {}", !is_cyclic_directed(&dag));

        let _nodes: Vec<_> = toposort(&dag, None)
            .unwrap()
            .into_iter()
            .map(|node_id| dag.node_weight(node_id).unwrap().as_str())
            .collect();

        let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");
        let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
        let leaf_node = dag
            .node_weight(*leaf_node_idx)
            .expect("unable to find node");

        assert_eq!(leaf_node, "close")
    }

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
        println!("{:?}", node);
        assert_eq!(node, "A");

        let leaf_node_idx = sorted_node_ids.get(1).expect("unable to get leaf");
        let node = dag.node_weight(*leaf_node_idx).unwrap();
        println!("{:?}", node);
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
        println!("{:?}", root_node);
        assert_eq!(root_node, strategy.score().calculation());
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
            println!("{:?}", node);
            if node == strategy.score().calculation() {
                break;
            }
        }
        assert_eq!(node, strategy.score().calculation());
    }

    #[test]
    fn dag_to_dot_file() {
        let mut dag: Dag = DiGraph::new();
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
    fn strategy_to_dag() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy).expect("unable to convert to bot");
        assert_eq!(dag.node_count(), 5);
        assert_eq!(dag.edge_count(), 6);
        let nodes = dag
            .node_indices()
            .map(|i| dag.node_weight(i).expect("node not found"))
            .find(|d| d.as_str().eq("sma200"))
            .expect("sma200 not found");
        assert_eq!(nodes, "sma200")
    }
}
