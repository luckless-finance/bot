use crate::dto::Strategy;
use petgraph::algo::{dijkstra, min_spanning_tree};
use petgraph::data::FromElements;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::{Directed, Graph};
use serde_yaml::to_string;
use std::borrow::Borrow;
use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::{BTreeMap, HashMap};
use petgraph::visit::{GraphBase, GraphRef};

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

pub fn to_dag(strategy: Strategy) -> DiGraph<String, String> {
    let strategy_yaml = serde_yaml::to_string(&strategy).expect("unable to string");
    println!("{}", strategy_yaml);
    let mut dag: DiGraph<String, String> = Graph::new();

    strategy.calculations().iter().for_each(|c| println!("{}", c.name()));
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
