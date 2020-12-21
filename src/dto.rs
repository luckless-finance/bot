#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html
type GenError = Box<dyn std::error::Error>;
type GenResult<T> = std::result::Result<T, GenError>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ScoreDto {
    calc: String,
}

impl ScoreDto {
    pub fn new(calc: String) -> Self {
        ScoreDto { calc }
    }
    pub fn calc(&self) -> &str {
        &self.calc
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum OperandType {
    Text,
    Integer,
    Decimal,
    Reference,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct OperandDto {
    name: String,
    #[serde(rename = "type")]
    _type: OperandType,
    value: String,
}

impl OperandDto {
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

impl OperandDto {
    pub fn new(name: String, _type: OperandType, value: String) -> Self {
        OperandDto { name, _type, value }
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum Operation {
    SMA,
    DIV,
    SUB,
    QUERY,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct CalculationDto {
    name: String,
    operation: Operation,
    operands: Vec<OperandDto>,
}

impl CalculationDto {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn operation(&self) -> &Operation {
        &self.operation
    }
    pub fn operands(&self) -> &Vec<OperandDto> {
        &self.operands
    }
}

impl CalculationDto {
    pub fn new(name: String, operation: Operation, operands: Vec<OperandDto>) -> Self {
        CalculationDto {
            name,
            operation,
            operands,
        }
    }
}

struct SmaCalculationDto {
    name: String,
    window_size: usize,
    time_series: String,
}

impl TryFrom<CalculationDto> for SmaCalculationDto {
    type Error = GenError;
    fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
        if calculation_dto.operation != Operation::SMA {
            Err(GenError::from("Conversion into SmaDto failed")).into()
        } else {
            let local_calculation_dto = calculation_dto.clone();
            let name: String = local_calculation_dto.name.clone();
            let window_size: usize = local_calculation_dto
                .operands
                .iter()
                .find(|o| o.name == "window_size")
                .ok_or("window_size is required")?
                .value
                .parse()?;
            let time_series: String = local_calculation_dto
                .operands
                .iter()
                .find(|o| o.name == "time_series")
                .ok_or("time_series is required")?
                .value
                .clone();
            Ok(Self {
                name,
                window_size,
                time_series,
            })
        }
    }
}

struct DivCalculationDto {
    name: String,
    denominator: String,
    numerator: String,
}

impl TryFrom<CalculationDto> for DivCalculationDto {
    type Error = GenError;
    fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
        if calculation_dto.operation != Operation::DIV {
            Err(GenError::from("Conversion into DivDto failed")).into()
        } else {
            let local_calculation_dto = calculation_dto.clone();
            let name: String = local_calculation_dto.name.clone();
            let numerator: String = local_calculation_dto
                .operands
                .iter()
                .find(|o| o.name == "numerator")
                .ok_or("numerator is required")?
                .value
                .parse()?;
            let denominator: String = local_calculation_dto
                .operands
                .iter()
                .find(|o| o.name == "denominator")
                .ok_or("denominator is required")?
                .value
                .clone();
            Ok(Self {
                name,
                numerator,
                denominator,
            })
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct StrategyDto {
    name: String,
    score: ScoreDto,
    calcs: Vec<CalculationDto>,
}

impl StrategyDto {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn score(&self) -> &ScoreDto {
        &self.score
    }
    pub fn calcs(&self) -> &Vec<CalculationDto> {
        &self.calcs
    }
}

impl StrategyDto {
    pub fn new(name: String, score: ScoreDto, calcs: Vec<CalculationDto>) -> Self {
        StrategyDto { name, score, calcs }
    }
}

pub(crate) fn from_path(file_path: &Path) -> Result<StrategyDto, serde_yaml::Error> {
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

    use crate::dto::*;
    use std::convert::TryInto;

    fn get_strategy() -> StrategyDto {
        StrategyDto {
            name: String::from("Example Strategy Document"),
            score: ScoreDto {
                calc: String::from("sma_gap"),
            },
            calcs: vec![
                CalculationDto {
                    name: String::from("sma_gap"),
                    operation: Operation::DIV,
                    operands: vec![
                        OperandDto {
                            name: String::from("numerator"),
                            _type: OperandType::Reference,
                            value: String::from("sma_diff"),
                        },
                        OperandDto {
                            name: String::from("denominator"),
                            _type: OperandType::Reference,
                            value: String::from("sma50"),
                        },
                    ],
                },
                CalculationDto {
                    name: String::from("sma_diff"),
                    operation: Operation::SUB,
                    operands: vec![
                        OperandDto {
                            name: String::from("left"),
                            _type: OperandType::Reference,
                            value: String::from("sma50"),
                        },
                        OperandDto {
                            name: String::from("right"),
                            _type: OperandType::Reference,
                            value: String::from("sma200"),
                        },
                    ],
                },
                CalculationDto {
                    name: String::from("sma50"),
                    operation: Operation::SMA,
                    operands: vec![
                        OperandDto {
                            name: String::from("window_size"),
                            _type: OperandType::Integer,
                            value: String::from("50"),
                        },
                        OperandDto {
                            name: String::from("time_series"),
                            _type: OperandType::Reference,
                            value: String::from("price"),
                        },
                    ],
                },
                CalculationDto {
                    name: String::from("sma200"),
                    operation: Operation::SMA,
                    operands: vec![
                        OperandDto {
                            name: String::from("window_size"),
                            _type: OperandType::Integer,
                            value: String::from("200"),
                        },
                        OperandDto {
                            name: String::from("time_series"),
                            _type: OperandType::Reference,
                            value: String::from("price"),
                        },
                    ],
                },
                CalculationDto {
                    name: String::from("price"),
                    operation: Operation::QUERY,
                    operands: vec![OperandDto {
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
        let actual_strategy: StrategyDto = serde_yaml::from_str(&actual_strategy_yaml)?;
        assert_eq!(actual_strategy, expected_strategy);
        Ok(())
    }

    #[test]
    fn yaml_to_strategy() -> Result<(), serde_yaml::Error> {
        let expected_strategy_yaml = get_strategy_yaml();
        // println!("{}", expected_strategy_yaml);
        let actual_strategy: StrategyDto =
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

    #[test]
    fn test_to_sma_dto() -> GenResult<()> {
        let x = r#"
name: sma200
operation: SMA
operands:
  - name: time_series
    type: Reference
    value: price
  - name: window_size
    type: Integer
    value: '200'"#;
        let calc_dto: CalculationDto = serde_yaml::from_str(x)?;
        let sma: SmaCalculationDto = calc_dto.try_into()?;
        assert_eq!(sma.name, "sma200");
        assert_eq!(sma.window_size, 200);
        assert_eq!(sma.time_series, "price");
        Ok(())
    }

    #[test]
    fn test_to_div_dto() -> GenResult<()> {
        let x = r#"
name: sma200
operation: DIV
operands:
  - name: denominator
    type: Reference
    value: foo
  - name: numerator
    type: Reference
    value: bar"#;
        let calc_dto: CalculationDto = serde_yaml::from_str(x)?;
        let sma: DivCalculationDto = calc_dto.try_into()?;
        assert_eq!(sma.name, "sma200");
        assert_eq!(sma.denominator, "foo");
        assert_eq!(sma.numerator, "bar");
        Ok(())
    }
}
