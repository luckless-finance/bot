use crate::dag::to_dag;
use crate::dto::{from_path, Strategy};
use std::env::current_dir;
use std::path::Path;

mod dag;
mod dto;

fn load_strategy() -> Strategy {
    let strategy_path = current_dir()
        .expect("unable to get working directory")
        .join(Path::new("strategy.yaml"));

    from_path(strategy_path.as_path()).expect("unable to load from file")
}

fn main() {
    println!(
        "current working directory: {}",
        current_dir()
            .expect("unable to get working directory")
            .to_str()
            .expect("unable to convert to str")
    );

    to_dag(load_strategy())
}
