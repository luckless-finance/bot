use std::env::current_dir;
use std::path::Path;
//
// use crate::dag::to_dag;
// use crate::dto::{from_path, Strategy};

// mod dag;
// mod data;
// mod dto;
// mod engine;
// mod foo;
mod dag_flow;
//
// fn load_strategy() -> Strategy {
//     let strategy_path = current_dir()
//         .expect("unable to get working directory")
//         .join(Path::new("strategy.yaml"));
//
//     from_path(strategy_path.as_path()).expect("unable to load from file")
// }
//
// fn demo_strategy() {
//     println!(
//         "current working directory: {}",
//         current_dir()
//             .expect("unable to get working directory")
//             .to_str()
//             .expect("unable to convert to str")
//     );
//     let dag = to_dag(&load_strategy()).expect("unable to convert to dag");
//     println!("{:?}", dag)
// }

fn main() {}
