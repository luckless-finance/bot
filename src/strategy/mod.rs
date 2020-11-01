use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Score {
    calculation: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Operand {
    name: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Calculation {
    name: String,
    operation: String,
    operands: Vec<Operand>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Strategy {
    name: String,
    score: Score,
    calculations: Vec<Calculation>,
}

#[cfg(test)]
mod tests {
    use crate::strategy::{Strategy, Score, Operand, Calculation};

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
}