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

#[deprecated]
pub fn to_dot_file_str_ref(g: &Graph<&str, &str, petgraph::Directed>) {
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

#[deprecated]
pub fn demo_dot() {
    let mut f: Graph<&str, &str, petgraph::Directed> = Graph::new();
    let tree_item1 = f.add_node("a");
    let tree_item2 = f.add_node("b");
    let tree_item3 = f.add_node("c");
    let tree_item4 = f.add_node("d");
    let tree_item5 = f.add_node("e");
    f.add_edge(tree_item1, tree_item2, "");
    f.add_edge(tree_item1, tree_item3, "");
    f.add_edge(tree_item2, tree_item4, "");
    f.add_edge(tree_item2, tree_item5, "");
    to_dot_file_str_ref(&f);
    //
    //     assert_eq!(f.edge_count(), 4);
    //     assert_eq!(f.node_count(), 5);
    //     let g = UnGraph::<i32, ()>::from_edges(&[
    //         (1, 2), (2, 3), (3, 4),
    //         (1, 4)]);
    //     let mst = UnGraph::<_, _>::from_elements(min_spanning_tree(&g));
    //     assert_eq!(g.raw_edges().len() - 1, mst.raw_edges().len());
    //
    // // Output the tree to `graphviz` `DOT` format
    //     println!("{:?}", Dot::with_config(&mst, &[Config::EdgeNoLabel]));
    //     ()
}

pub struct StrategyDag {
    strategy: Strategy,
    dag: Graph<String, String, petgraph::Directed>,
}

impl StrategyDag {
    pub fn strategy(a: &StrategyDag) -> &Strategy {
        &a.strategy
    }
    pub fn dag(a: &StrategyDag) -> &Graph<String, String, petgraph::Directed> {
        &a.dag
    }
}

pub fn to_dag(strategy: Strategy) {
    let strategy_yaml = serde_yaml::to_string(&strategy).expect("unable to string");
    println!("{}", strategy_yaml);
    let mut dag: Graph<String, String, petgraph::Directed> = Graph::new();




    to_dot_file(&dag);
    ()
}
