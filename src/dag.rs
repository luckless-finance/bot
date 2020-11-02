use crate::dto::Strategy;
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::{GraphBase, GraphRef};
use petgraph::{Directed, Graph};
use serde_yaml::to_string;
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn to_dot_file(g: &Graph<String, String, petgraph::Directed>) {
    let mut output_file = File::create(
        current_dir()
            .expect("unable to find current_dir")
            .join("output.dot"),
    )
    .expect("unable to open output file");
    // let mut dot_text = String::new();
    let dot_text = format!("{:?}", Dot::with_config(g, &[Config::EdgeNoLabel]));
    output_file
        .write_all(dot_text.as_bytes())
        .expect("unable to write file");
}

pub fn to_dag(strategy: &Strategy) -> DiGraph<String, String> {
    let strategy_yaml = serde_yaml::to_string(&strategy).expect("unable to string");
    println!("{}", strategy_yaml);
    let mut dag: DiGraph<String, String> = Graph::new();

    strategy
        .calculations()
        .iter()
        .for_each(|c| println!("{}", c.name()));
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

    to_dot_file(&dag);
    dag
}

#[cfg(test)]
mod tests {
    use crate::dag::{to_dag, to_dot_file};
    use crate::dto::from_path;
    use petgraph::prelude::DiGraph;
    use petgraph::Graph;
    use std::borrow::Borrow;
    use std::fs::read_to_string;
    use std::path::Path;

    #[test]
    fn to_dot_file_test() {
        let mut dag: DiGraph<String, String> = Graph::new();
        let node_index_A = dag.add_node(String::from("A"));
        let node_index_B = dag.add_node(String::from("B"));
        let node_index_C = dag.add_node(String::from("C"));
        let node_index_D = dag.add_node(String::from("D"));
        dag.add_edge(node_index_A, node_index_B, String::from(""));
        dag.add_edge(node_index_A, node_index_C, String::from(""));
        dag.add_edge(node_index_B, node_index_D, String::from(""));
        dag.add_edge(node_index_C, node_index_D, String::from(""));
        to_dot_file(&dag);
        let expected_output =
            read_to_string("expected_output.dot").expect("expected_output.dot not found.");
        let output = read_to_string("output.dot").expect("output.dot not found.");
        assert_eq!(output, expected_output);
    }

    #[test]
    fn to_dag_test() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy);
        assert_eq!(dag.node_count(), 5);
        assert_eq!(dag.edge_count(), 6);
        let nodes = dag
            .node_indices()
            .map(|i| dag.node_weight(i).expect("node not found"))
            .find(|d| d.as_str().eq("sma200"))
            .expect("sma200 not found");
        print!("{}", nodes);
    }
}
