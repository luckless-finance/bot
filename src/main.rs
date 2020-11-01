mod dto;

fn main() {
    let expected_strategy_yaml = String::from(r#"---
name: foo
score:
  calculation: bar
calculations:
  - name: calc
    operation: add
    operands:
      - name: operand"#);

    let actual_strategy: dto::Strategy = serde_yaml::from_str(&expected_strategy_yaml)
        .expect("unable to parse yaml");
    let actual_strategy_yaml = serde_yaml::to_string(&actual_strategy)
        .expect("unable to parse yaml");

    println!("{}", expected_strategy_yaml);
    println!("{}", actual_strategy_yaml);
}
