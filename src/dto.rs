use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::borrow::BorrowMut;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    calculation: String
}

impl Score {
    pub fn new(calculation: String) -> Self {
        Score { calculation }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Operand {
    name: String
}

impl Operand {
    pub fn new(name: String) -> Self {
        Operand { name }
    }
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Calculation {
    name: String,
    operation: String,
    operands: Vec<Operand>,
}

impl Calculation {
    pub fn new(name: String, operation: String, operands: Vec<Operand>) -> Self {
        Calculation { name, operation, operands }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    name: String,
    score: Score,
    calculations: Vec<Calculation>,
}

impl Strategy {
    pub fn new(name: String, score: Score, calculations: Vec<Calculation>) -> Self {
        Strategy { name, score, calculations }
    }
}

pub fn from_path(file_path: &Path) -> Result<Strategy, serde_yaml::Error> {
    let mut strategy_file: File = File::open(file_path)
        .expect("unable to open file");
    let mut strategy_yaml = String::new();
    strategy_file.read_to_string(strategy_yaml.borrow_mut());
    serde_yaml::from_str(&strategy_yaml)
}

#[cfg(test)]
mod tests {
    use crate::dto::{Strategy, Score, Operand, Calculation, from_path};
    use std::env::current_dir;
    use std::path::Path;

    fn get_strategy() -> Strategy {
        Strategy {
            name: String::from("foo"),
            score: Score {
                calculation: String::from("bar")
            },
            calculations: vec![
                Calculation {
                    name: String::from("calc"),
                    operation: String::from("add"),
                    operands: vec![
                        Operand {
                            name: String::from("operand")
                        }
                    ],
                }
            ],
        }
    }

    fn get_strategy_yaml() -> String {
        String::from(r#"---
name: foo
score:
  calculation: bar
calculations:
  - name: calc
    operation: add
    operands:
      - name: operand"#)
    }

    #[test]
    fn strategy_dto_test() {
        let s = get_strategy();
        assert_eq!(s.name, "foo");
        assert_eq!(s.score.calculation, "bar");
        assert_eq!(s.calculations[0].name, "calc");
        assert_eq!(s.calculations[0].operation, "add");
        assert_eq!(s.calculations[0].operands[0].name, "operand");
    }

    #[test]
    fn strategy_to_yaml() -> Result<(), serde_yaml::Error> {
        let expected_strategy = get_strategy();
        let actual_strategy_yaml = serde_yaml::to_string(&expected_strategy)?;
        let actual_strategy: Strategy = serde_yaml::from_str(&actual_strategy_yaml)?;
        assert_eq!(actual_strategy, expected_strategy);
        Ok(())
    }

    #[test]
    fn yaml_to_strategy() -> Result<(), serde_yaml::Error> {
        let expected_strategy_yaml = get_strategy_yaml();
        let actual_strategy: Strategy = serde_yaml::from_str(&expected_strategy_yaml)?;
        let actual_strategy_yaml: String = serde_yaml::to_string(&actual_strategy)?;
        assert_eq!(actual_strategy_yaml, expected_strategy_yaml);
        Ok(())
    }

    #[test]
    fn constructors() {
        let strategy = Strategy::new(
            String::from("foo"),
            Score::new(String::from("bar")),
            vec![
                Calculation::new(
                    String::from("calc"),
                    String::from("add"),
                    vec![
                        Operand::new(String::from("operand")),
                    ]),
            ],
        );
        assert_eq!(strategy, get_strategy());
    }

    #[test]
    fn test_from_file() {
        let mut strategy_path = current_dir()
            .expect("unable to get working directory")
            .join(Path::new("strategy.yaml"));
        let strategy = from_path(strategy_path.as_path()).expect("unable to load strategy from path");
        assert_eq!(strategy, get_strategy());
    }
}
