use crate::dto::{from_path, Strategy};
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::path::Path;

mod dto;

fn main() {
  println!(
    "current working directory: {}",
    current_dir()
      .expect("unable to get working directory")
      .to_str()
      .expect("unable to convert to str")
  );
  let strategy_path = current_dir()
    .expect("unable to get working directory")
    .join(Path::new("strategy.yaml"));

  let strategy: Strategy = from_path(strategy_path.as_path()).expect("unable to load from file");
  let strategy_yaml = serde_yaml::to_string(&strategy).expect("unable to parse yaml");

  let expected_strategy_yaml = String::from(
    r#"---
name: foo
score:
  calculation: bar
calculations:
  - name: calc
    operation: add
    operands:
      - name: operand"#,
  );

  let actual_strategy: dto::Strategy =
    serde_yaml::from_str(&expected_strategy_yaml).expect("unable to parse yaml");
  let actual_strategy_yaml = serde_yaml::to_string(&actual_strategy).expect("unable to parse yaml");

  println!("{}", expected_strategy_yaml);
  println!("{}", actual_strategy_yaml);
}
