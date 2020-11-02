use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    calculation: String,
}

impl Score {
    pub fn new(calculation: String) -> Self {
        Score { calculation }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Operand {
    name: String,
    _type: String,
    value: String,
}

impl Operand {
    pub fn new(name: String, _type: String, value: String) -> Self {
        Operand { name, _type, value }
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
        Calculation {
            name,
            operation,
            operands,
        }
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
        Strategy {
            name,
            score,
            calculations,
        }
    }
}

pub fn from_path(file_path: &Path) -> Result<Strategy, serde_yaml::Error> {
    let mut strategy_file: File = File::open(file_path).expect("unable to open file");
    let mut strategy_yaml = String::new();
    strategy_file
        .read_to_string(strategy_yaml.borrow_mut())
        .expect("unable to read strategy file");
    serde_yaml::from_str(&strategy_yaml)
}

#[cfg(test)]
mod tests {
    use crate::dto::{from_path, Calculation, Operand, Score, Strategy};
    use std::env::current_dir;
    use std::path::Path;

    fn get_strategy() -> Strategy {
        Strategy {
            name: String::from("Example Strategy Document"),
            score: Score {
                calculation: String::from("sma_gap"),
            },
            calculations: vec![
                Calculation {
                    name: String::from("sma_gap"),
                    operation: String::from("div"),
                    operands: vec![
                        Operand {
                            name: String::from("numerator"),
                            _type: String::from("ref"),
                            value: String::from("sma_diff"),
                        },
                        Operand {
                            name: String::from("denominator"),
                            _type: String::from("ref"),
                            value: String::from("sma50"),
                        },
                    ],
                },
                Calculation {
                    name: String::from("sma_diff"),
                    operation: String::from("sub"),
                    operands: vec![
                        Operand {
                            name: String::from("left"),
                            _type: String::from("ref"),
                            value: String::from("sma50"),
                        },
                        Operand {
                            name: String::from("right"),
                            _type: String::from("ref"),
                            value: String::from("sma200"),
                        },
                    ],
                },
                Calculation {
                    name: String::from("sma50"),
                    operation: String::from("sma"),
                    operands: vec![
                        Operand {
                            name: String::from("window_size"),
                            _type: String::from("i32"),
                            value: String::from("50"),
                        },
                        Operand {
                            name: String::from("sequence"),
                            _type: String::from("query"),
                            value: String::from("close"),
                        },
                    ],
                },
                Calculation {
                    name: String::from("sma200"),
                    operation: String::from("sma"),
                    operands: vec![
                        Operand {
                            name: String::from("window_size"),
                            _type: String::from("i32"),
                            value: String::from("200"),
                        },
                        Operand {
                            name: String::from("sequence"),
                            _type: String::from("query"),
                            value: String::from("close"),
                        },
                    ],
                },
            ],
        }
    }

    fn get_strategy_yaml() -> String {
        String::from(
            r#"---
name: Example Strategy Document
score:
  calculation: sma_gap
calculations:
  - name: sma_gap
    operation: div
    operands:
      - name: numerator
        _type: ref
        value: sma_diff
      - name: denominator
        _type: ref
        value: sma50
  - name: sma_diff
    operation: sub
    operands:
      - name: left
        _type: ref
        value: sma50
      - name: right
        _type: ref
        value: sma200
  - name: sma50
    operation: sma
    operands:
      - name: window_size
        _type: i32
        value: 50
      - name: sequence
        _type: query
        value: close
  - name: sma200
    operation: sma
    operands:
      - name: window_size
        _type: i32
        value: 200
      - name: sequence
        _type: query
        value: close"#,
        )
    }

    #[test]
    fn constructors() {
        let s = get_strategy();
        assert_eq!(s.name, "Example Strategy Document");
        assert_eq!(s.score.calculation, "sma_gap");
        assert_eq!(s.calculations[0].name, "sma_gap");
        assert_eq!(s.calculations[0].operation, "div");
        assert_eq!(s.calculations[0].operands[0].name, "numerator");
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
        println!("{}", expected_strategy_yaml);
        let actual_strategy: Strategy =
            serde_yaml::from_str(&expected_strategy_yaml).expect("unable to parse yaml");
        // let actual_strategy_yaml: String = serde_yaml::to_string(&actual_strategy)?;
        // assert_eq!(actual_strategy_yaml, expected_strategy_yaml);
        Ok(())
    }

    #[test]
    fn test_from_file() {
        let mut strategy_path = current_dir()
            .expect("unable to get working directory")
            .join(Path::new("strategy.yaml"));
        let strategy =
            from_path(strategy_path.as_path()).expect("unable to load strategy from path");
        assert_eq!(strategy, get_strategy());
    }
}
