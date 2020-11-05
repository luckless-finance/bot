use std::collections::HashMap;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;

use petgraph::Graph;
use petgraph::algo::{
    connected_components, is_cyclic_directed,
};
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;

use crate::dto::Strategy;

// type Dag = Graph<String, String, petgraph::Directed>;
pub type Dag = DiGraph<String, String>;

pub fn to_dot_text(g: &Dag) -> String {
    format!("{:?}", Dot::with_config(g, &[Config::EdgeNoLabel]))
}

pub fn to_dot_file(g: &Dag) {
    let mut output_file = File::create(
        current_dir()
            .expect("unable to find current_dir")
            .join("output.dot"),
    ).expect("unable to open output file");
    let dot_text = to_dot_text(g);
    output_file
        .write_all(dot_text.as_bytes())
        .expect("unable to write file");
}

pub fn to_dag(strategy: &Strategy) -> Result<Dag, &str> {
    let mut dag: Dag = Graph::new();
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
    use std::fs::read_to_string;
    use std::path::Path;

    use petgraph::algo::{ toposort};
    use petgraph::prelude::DiGraph;

    use crate::dag::{to_dag, to_dot_file};
    use crate::dto::from_path;
    type Dag = DiGraph<String, String>;


    #[test]
    fn get_queries() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let mut dag = to_dag(&strategy).expect("unable to convert to dag");
        // let A = dag.add_node(String::from("A"));
        // let B = dag.add_node(String::from("B"));
        // let C = dag.add_node(String::from("C"));
        // dag.add_edge(A, B, String::new());
        // dag.add_edge(C, B, String::new());
        to_dot_file(&dag);
        // println!("is a dag? {}", !is_cyclic_directed(&dag));

        let nodes: Vec<_> = toposort(&dag, None)
            .unwrap()
            .into_iter()
            .map(|node_id| dag.node_weight(node_id).unwrap().as_str())
            .collect();

        println!("{:?}", nodes);

        let topo_node_ids = toposort(&dag, None).expect("unable to toposort");
        let root_node_id = topo_node_ids.get(0).expect("unable to get root");
        let root_node = dag.node_weight(*root_node_id).expect("unable to find node");

        assert_eq!(root_node, "close")
    }

    #[test]
    fn to_dot_file_test() {
        let mut dag: Dag = DiGraph::new();
        let node_index_a = dag.add_node(String::from("A"));
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
    fn to_dag_test() {
        let strategy = from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
        let dag = to_dag(&strategy).expect("unable to convert to dag");
        assert_eq!(dag.node_count(), 5);
        assert_eq!(dag.edge_count(), 6);
        let nodes = dag
            .node_indices()
            .map(|i| dag.node_weight(i).expect("node not found"))
            .find(|d| d.as_str().eq("sma200"))
            .expect("sma200 not found");
        // print!("{}", nodes);
    }
}
