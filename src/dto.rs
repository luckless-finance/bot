#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ScoreDTO {
    calc: String,
}

impl ScoreDTO {
    pub fn new(calc: String) -> Self {
        ScoreDTO { calc }
    }
    pub fn calc(&self) -> &str {
        &self.calc
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum OperandType {
    Text,
    Integer,
    Decimal,
    Reference,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OperandDTO {
    name: String,
    #[serde(rename = "type")]
    _type: OperandType,
    value: String,
}

impl OperandDTO {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn _type(&self) -> &OperandType {
        &self._type
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl OperandDTO {
    pub fn new(name: String, _type: OperandType, value: String) -> Self {
        OperandDTO { name, _type, value }
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Operation {
    SMA,
    DIV,
    SUB,
    QUERY,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CalculationDTO {
    name: String,
    operation: Operation,
    operands: Vec<OperandDTO>,
}

impl CalculationDTO {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn operation(&self) -> &Operation {
        &self.operation
    }
    pub fn operands(&self) -> &Vec<OperandDTO> {
        &self.operands
    }
}

impl CalculationDTO {
    pub fn new(name: String, operation: Operation, operands: Vec<OperandDTO>) -> Self {
        CalculationDTO {
            name,
            operation,
            operands,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StrategyDTO {
    name: String,
    score: ScoreDTO,
    calcs: Vec<CalculationDTO>,
}

impl StrategyDTO {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn score(&self) -> &ScoreDTO {
        &self.score
    }
    pub fn calcs(&self) -> &Vec<CalculationDTO> {
        &self.calcs
    }
}

impl StrategyDTO {
    pub fn new(name: String, score: ScoreDTO, calcs: Vec<CalculationDTO>) -> Self {
        StrategyDTO { name, score, calcs }
    }
}

pub fn from_path(file_path: &Path) -> Result<StrategyDTO, serde_yaml::Error> {
    let mut strategy_file: File = File::open(file_path).expect("unable to open file");
    let mut strategy_yaml = String::new();
    strategy_file
        .read_to_string(strategy_yaml.borrow_mut())
        .expect("unable to read strategy file");
    serde_yaml::from_str(&strategy_yaml)
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;
    use std::path::Path;

    use crate::dto::{
        from_path, CalculationDTO, OperandDTO, OperandType, Operation, ScoreDTO, StrategyDTO,
    };

    fn get_strategy() -> StrategyDTO {
        StrategyDTO {
            name: String::from("Example Strategy Document"),
            score: ScoreDTO {
                calc: String::from("sma_gap"),
            },
            calcs: vec![
                CalculationDTO {
                    name: String::from("sma_gap"),
                    operation: Operation::DIV,
                    operands: vec![
                        OperandDTO {
                            name: String::from("numerator"),
                            _type: OperandType::Reference,
                            value: String::from("sma_diff"),
                        },
                        OperandDTO {
                            name: String::from("denominator"),
                            _type: OperandType::Reference,
                            value: String::from("sma50"),
                        },
                    ],
                },
                CalculationDTO {
                    name: String::from("sma_diff"),
                    operation: Operation::SUB,
                    operands: vec![
                        OperandDTO {
                            name: String::from("left"),
                            _type: OperandType::Reference,
                            value: String::from("sma50"),
                        },
                        OperandDTO {
                            name: String::from("right"),
                            _type: OperandType::Reference,
                            value: String::from("sma200"),
                        },
                    ],
                },
                CalculationDTO {
                    name: String::from("sma50"),
                    operation: Operation::SMA,
                    operands: vec![
                        OperandDTO {
                            name: String::from("window_size"),
                            _type: OperandType::Integer,
                            value: String::from("50"),
                        },
                        OperandDTO {
                            name: String::from("time_series"),
                            _type: OperandType::Reference,
                            value: String::from("price"),
                        },
                    ],
                },
                CalculationDTO {
                    name: String::from("sma200"),
                    operation: Operation::SMA,
                    operands: vec![
                        OperandDTO {
                            name: String::from("window_size"),
                            _type: OperandType::Integer,
                            value: String::from("200"),
                        },
                        OperandDTO {
                            name: String::from("time_series"),
                            _type: OperandType::Reference,
                            value: String::from("price"),
                        },
                    ],
                },
                CalculationDTO {
                    name: String::from("price"),
                    operation: Operation::QUERY,
                    operands: vec![OperandDTO {
                        name: String::from("field"),
                        _type: OperandType::Text,
                        value: String::from("close"),
                    }],
                },
            ],
        }
    }

    fn get_strategy_yaml() -> String {
        String::from(
            r#"---
name: Example Strategy Document
score:
  calc: sma_gap
calcs:
  - name: sma_gap
    operation: DIV
    operands:
      - name: numerator
        type: Reference
        value: sma_diff
      - name: denominator
        type: Reference
        value: sma50
  - name: sma_diff
    operation: SUB
    operands:
      - name: left
        type: Reference
        value: sma50
      - name: right
        type: Reference
        value: sma200
  - name: sma50
    operation: SMA
    operands:
      - name: window_size
        type: Integer
        value: "50"
      - name: time_series
        type: Reference
        value: price
  - name: sma200
    operation: SMA
    operands:
      - name: window_size
        type: Integer
        value: "200"
      - name: time_series
        type: Reference
        value: price
  - name: price
    operation: QUERY
    operands:
      - name: field
        type: Text
        value: close"#,
        )
    }

    #[test]
    fn constructors() {
        let s = get_strategy();
        assert_eq!(s.name, "Example Strategy Document");
        assert_eq!(s.score.calc, "sma_gap");
        assert_eq!(s.calcs[0].name, "sma_gap");
        assert_eq!(s.calcs[0].operation, Operation::DIV);
        assert_eq!(s.calcs[0].operands[0].name, "numerator");
    }

    #[test]
    fn strategy_to_yaml() -> Result<(), serde_yaml::Error> {
        let expected_strategy = get_strategy();
        let actual_strategy_yaml = serde_yaml::to_string(&expected_strategy)?;
        let actual_strategy: StrategyDTO = serde_yaml::from_str(&actual_strategy_yaml)?;
        assert_eq!(actual_strategy, expected_strategy);
        Ok(())
    }

    #[test]
    fn yaml_to_strategy() -> Result<(), serde_yaml::Error> {
        let expected_strategy_yaml = get_strategy_yaml();
        // println!("{}", expected_strategy_yaml);
        let actual_strategy: StrategyDTO =
            serde_yaml::from_str(&expected_strategy_yaml).expect("unable to parse yaml");
        let actual_strategy_yaml: String = serde_yaml::to_string(&actual_strategy)?;
        assert_eq!(actual_strategy_yaml, expected_strategy_yaml);
        Ok(())
    }

    #[test]
    fn test_from_file() {
        let strategy_path = current_dir()
            .expect("unable to get working directory")
            .join(Path::new("strategy.yaml"));
        let strategy =
            from_path(strategy_path.as_path()).expect("unable to load strategy from path");
        assert_eq!(strategy, get_strategy());
    }
}
