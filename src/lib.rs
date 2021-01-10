#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(map_first_last)]
#![feature(array_zip)]
#![feature(iter_advance_by)]
#![feature(core_panic)]
#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod data;
pub mod errors;
pub mod simulation;
pub mod time_series;

pub mod bot {
    pub mod asset_score {
        #![allow(dead_code)]

        use std;
        use std::collections::HashMap;
        use std::convert::TryInto;
        use std::fmt;

        use serde::export::Formatter;

        use crate::bot::dag::Dag;
        use crate::data::{Asset, DataClient};
        use crate::dto::strategy::{
            CalculationDto, DyadicScalarCalculationDto, DyadicTsCalculationDto, Operation,
            QueryCalculationDto, SmaCalculationDto, StrategyDto, TimeSeriesName,
        };
        use crate::errors::{GenResult, UpstreamNotFoundError};
        use crate::time_series::{DataPointValue, TimeSeries1D, TimeStamp};

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum CalculationStatus {
            NotStarted,
            InProgress,
            Complete,
            Error,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum AssetScoreStatus {
            NotStarted,
            InProgress,
            Complete,
            Error,
        }

        /// Wraps several Dtos required traverse and consume a strategy
        #[derive(Debug, Clone)]
        pub struct CompiledStrategy {
            strategy: StrategyDto,
            dag: Dag,
            calcs: HashMap<TimeSeriesName, CalculationDto>,
        }

        impl CompiledStrategy {
            pub fn new(strategy: StrategyDto) -> GenResult<Self> {
                let dag = Dag::new(strategy.clone())?;
                let calcs: HashMap<String, CalculationDto> = strategy
                    .calcs()
                    .iter()
                    .map(|calc| (calc.name().to_string(), calc.clone()))
                    .collect();
                Ok(CompiledStrategy {
                    strategy,
                    dag,
                    calcs,
                })
            }
            /// Computes the score of the given `Asset` at the given `TimeStamp`
            pub fn asset_score(
                &self,
                asset: Asset,
                timestamp: TimeStamp,
                data_client: Box<dyn DataClient>,
            ) -> GenResult<AssetScore> {
                let mut scorable_asset = ScorableAsset {
                    asset,
                    timestamp,
                    execution_order: self.dag.execution_order().clone(),
                    calcs: self.calcs.clone(),
                    data_client,
                    calc_status: self
                        .calcs
                        .keys()
                        .map(|c| (c.clone(), CalculationStatus::NotStarted))
                        .collect(),
                    calc_time_series: HashMap::new(),
                };
                scorable_asset.execute()?;
                Ok(AssetScore::new(scorable_asset)?)
            }
        }

        /// Composes a `Bot` with a `Asset`, `Timestamp` and `DataClient`.
        #[derive(Debug)]
        pub struct ScorableAsset {
            asset: Asset,
            timestamp: TimeStamp,
            execution_order: Vec<TimeSeriesName>,
            calcs: HashMap<TimeSeriesName, CalculationDto>,
            data_client: Box<dyn DataClient>,
            calc_status: HashMap<TimeSeriesName, CalculationStatus>,
            calc_time_series: HashMap<TimeSeriesName, TimeSeries1D>,
        }

        impl ScorableAsset {
            pub(crate) fn overall_status(&self) -> AssetScoreStatus {
                // compute group by count using Entry Api
                let mut count_by_status: HashMap<CalculationStatus, usize> = HashMap::new();
                for calc_status in self.calc_status.values().into_iter() {
                    let count = count_by_status.entry(calc_status.clone()).or_insert(0usize);
                    *count += 1;
                }
                // declare determining factors of overall status
                let has_error = match count_by_status.get(&CalculationStatus::Error) {
                    Some(_) => true,
                    None => false,
                };
                let all_complete = match count_by_status.get(&CalculationStatus::Complete) {
                    Some(n) => n == &self.calcs.len(),
                    None => false,
                };
                let all_not_started = match count_by_status.get(&CalculationStatus::NotStarted) {
                    Some(n) => n == &self.calcs.len(),
                    None => false,
                };
                // apply business logic against factors
                if has_error {
                    AssetScoreStatus::Error
                } else if all_complete {
                    AssetScoreStatus::Complete
                } else if all_not_started {
                    AssetScoreStatus::NotStarted
                } else {
                    AssetScoreStatus::InProgress
                }
            }
            fn status(&mut self, calc_name: &str, new_calc_status: CalculationStatus) {
                if let Some(calc_status) = self.calc_status.get_mut(calc_name) {
                    *calc_status = new_calc_status;
                }
            }

            pub fn upstream(&self, calc_name: &str) -> GenResult<&TimeSeries1D> {
                match self.calc_time_series.get(calc_name) {
                    Some(time_series_) => Ok(time_series_),
                    None => Err(UpstreamNotFoundError::new(calc_name.to_string())),
                }
            }

            pub(crate) fn score(&self) -> GenResult<&TimeSeries1D> {
                Ok(self.upstream(self.execution_order.last().expect("impossible"))?)
            }

            /// Traverse `Dag` executing each node for given `Asset` as of `Timestamp`
            fn execute(&mut self) -> GenResult<()> {
                let calc_order = self.execution_order.clone();
                for calc_name in calc_order {
                    // println!("\nexecuting {}", calc_name);
                    self.status(&calc_name, CalculationStatus::InProgress);
                    let calc = self.calcs.get(&calc_name).ok_or("calc not found")?;

                    let calc_time_series = match calc.operation() {
                        Operation::QUERY => self.handle_query(calc),
                        Operation::ADD => self.handle_add(calc),
                        Operation::SUB => self.handle_sub(calc),
                        Operation::MUL => self.handle_mul(calc),
                        Operation::DIV => self.handle_div(calc),
                        Operation::TS_ADD => self.handle_ts_add(calc),
                        Operation::TS_SUB => self.handle_ts_sub(calc),
                        Operation::TS_MUL => self.handle_ts_mul(calc),
                        Operation::TS_DIV => self.handle_ts_div(calc),
                        Operation::SMA => self.handle_sma(calc),
                    };
                    self.status(
                        &calc_name,
                        match calc_time_series.is_ok() {
                            true => CalculationStatus::Complete,
                            false => CalculationStatus::Error,
                        },
                    );

                    self.calc_time_series
                        .insert(calc_name.clone(), calc_time_series?);
                }
                Ok(())
            }
            // TODO parameterized query: generalize market data retrieval
            fn handle_query(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::QUERY);
                let query_dto: QueryCalculationDto = calculation_dto.clone().try_into()?;
                Ok(self
                    .data_client
                    .query(&self.asset, &self.timestamp, Some(query_dto))?
                    .clone())
            }
            fn handle_add(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::ADD);
                let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
                    calculation_dto.clone().try_into()?;
                let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
                Ok(time_series.add(dyadic_scalar_calc_dto.scalar()))
            }
            fn handle_sub(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::SUB);
                let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
                    calculation_dto.clone().try_into()?;
                let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
                Ok(time_series.sub(dyadic_scalar_calc_dto.scalar()))
            }
            fn handle_mul(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::MUL);
                let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
                    calculation_dto.clone().try_into()?;
                let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
                Ok(time_series.mul(dyadic_scalar_calc_dto.scalar()))
            }
            fn handle_div(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::DIV);
                let dyadic_scalar_calc_dto: DyadicScalarCalculationDto =
                    calculation_dto.clone().try_into()?;
                let time_series = self.upstream(dyadic_scalar_calc_dto.time_series())?;
                Ok(time_series.div(dyadic_scalar_calc_dto.scalar()))
            }
            fn handle_ts_add(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::TS_ADD);
                let dyadic_ts_calc_dto: DyadicTsCalculationDto =
                    calculation_dto.clone().try_into()?;
                let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
                let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
                Ok(left_value.ts_add(right_value))
            }
            fn handle_ts_sub(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::TS_SUB);
                let dyadic_ts_calc_dto: DyadicTsCalculationDto =
                    calculation_dto.clone().try_into()?;
                let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
                let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
                Ok(left_value.ts_sub(right_value))
            }
            fn handle_ts_mul(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::TS_MUL);
                let dyadic_ts_calc_dto: DyadicTsCalculationDto =
                    calculation_dto.clone().try_into()?;
                let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
                let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
                Ok(left_value.ts_mul(right_value))
            }
            fn handle_ts_div(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::TS_DIV);
                let dyadic_ts_calc_dto: DyadicTsCalculationDto =
                    calculation_dto.clone().try_into()?;
                let left_value = self.upstream(dyadic_ts_calc_dto.left())?;
                let right_value = self.upstream(dyadic_ts_calc_dto.right())?;
                Ok(left_value.ts_div(right_value))
            }
            fn handle_sma(&self, calculation_dto: &CalculationDto) -> GenResult<TimeSeries1D> {
                assert_eq!(*calculation_dto.operation(), Operation::SMA);
                let sma_dto: SmaCalculationDto = calculation_dto.clone().try_into()?;
                let time_series = self.upstream(sma_dto.time_series())?;
                Ok(time_series.sma(sma_dto.window_size()))
            }
        }

        #[derive(Debug)]
        pub struct AssetScore {
            asset: Asset,
            timestamp: TimeStamp,
            score: TimeSeries1D,
            status: AssetScoreStatus,
        }

        impl AssetScore {
            fn new(scorable_asset: ScorableAsset) -> GenResult<AssetScore> {
                // TODO warn when overall_status is not Complete
                let status = scorable_asset.overall_status();
                let score = scorable_asset.score()?.clone();
                Ok(AssetScore {
                    asset: scorable_asset.asset,
                    timestamp: scorable_asset.timestamp,
                    score,
                    status,
                })
            }
            pub fn asset(&self) -> &Asset {
                &self.asset
            }
            fn timestamp(&self) -> &TimeStamp {
                &self.timestamp
            }
            pub fn score(&self) -> &TimeSeries1D {
                &self.score
            }
            fn status(&self) -> &AssetScoreStatus {
                &self.status
            }
        }

        #[cfg(test)]
        mod tests {
            use std::collections::HashMap;
            use std::path::Path;

            use crate::bot::asset_score::{
                AssetScore, AssetScoreStatus, CalculationStatus, CompiledStrategy,
            };
            use crate::data::Asset;
            use crate::dto::strategy::{
                from_path, CalculationDto, OperandDto, OperandType, Operation, ScoreDto,
                StrategyDto,
            };
            use crate::errors::GenResult;
            use crate::simulation::MockDataClient;

            fn strategy_fixture() -> StrategyDto {
                StrategyDto::new(
                    String::from("Small Strategy Document"),
                    ScoreDto::new(String::from("price")),
                    vec![CalculationDto::new(
                        String::from("price"),
                        Operation::QUERY,
                        vec![OperandDto::new(
                            String::from("field"),
                            OperandType::Text,
                            String::from("close"),
                        )],
                    )],
                )
            }

            fn compiled_strategy_fixture() -> GenResult<CompiledStrategy> {
                let strategy =
                    from_path(Path::new("strategy.yaml")).expect("unable to load strategy");
                CompiledStrategy::new(strategy)
            }

            #[test]
            fn asset_score() -> GenResult<()> {
                let compiled_strategy = compiled_strategy_fixture()?;
                let asset = Asset::new(String::from("A"));
                let timestamp = MockDataClient::today();
                let data_client = MockDataClient::new();
                let asset_score: AssetScore =
                    compiled_strategy.asset_score(asset, timestamp, Box::new(data_client))?;
                assert_eq!(asset_score.status, AssetScoreStatus::Complete);
                Ok(())
            }
        }
    }

    pub mod dag {
        #![allow(dead_code)]

        use core::fmt;
        use std::collections::HashMap;
        use std::convert::{TryFrom, TryInto};
        use std::env::current_dir;
        use std::fmt::Formatter;
        use std::fs::File;
        use std::io::Write;

        use petgraph::algo::{connected_components, is_cyclic_directed, toposort};
        use petgraph::dot::{Config, Dot};
        use petgraph::graph::{DiGraph, NodeIndex};
        use petgraph::Direction;

        use crate::dto::strategy::{OperandType, StrategyDto};
        use crate::errors::{GenError, GenResult, InvalidStrategyError};

        /// Directed acyclic graph where vertices/nodes represent calculations and edges represent dependencies.
        #[derive(Debug, Clone)]
        pub struct Dag {
            dag_dto: DiGraph<String, String>,
            node_lkup: HashMap<String, NodeIndex>,
        }

        impl Dag {
            pub fn new(strategy_dto: StrategyDto) -> GenResult<Self> {
                let dag_dto: DiGraph<String, String> = strategy_dto.try_into()?;
                let node_lkup: HashMap<String, NodeIndex<u32>> = dag_dto
                    .node_indices()
                    .into_iter()
                    .map(|idx| (dag_dto.node_weight(idx).expect("impossible").clone(), idx))
                    .collect();
                Ok(Dag { dag_dto, node_lkup })
            }
            pub fn execution_order(&self) -> Vec<String> {
                toposort(&self.dag_dto, None)
                    .expect("unable to toposort")
                    .iter()
                    .map(|node_idx: &NodeIndex| {
                        self.dag_dto.node_weight(*node_idx).unwrap().clone()
                    })
                    .collect()
            }
            pub fn upstream(&self, node: &String) -> Vec<String> {
                self.dag_dto
                    .neighbors_directed(
                        self.node_lkup.get(node).expect("node not found").clone(),
                        Direction::Incoming,
                    )
                    .map(|x| self.dag_dto.node_weight(x).expect("node not found").clone())
                    .collect()
            }
            fn save_dot_file(&self) {
                let mut output_file = File::create(
                    current_dir()
                        .expect("unable to find current_dir")
                        .join("output.dot"),
                )
                .expect("unable to open output file");
                let dot_text = format!(
                    "{:?}",
                    Dot::with_config(&self.dag_dto, &[Config::EdgeNoLabel])
                );
                output_file
                    .write_all(dot_text.as_bytes())
                    .expect("unable to write file");
            }
        }

        impl TryFrom<StrategyDto> for DiGraph<String, String> {
            type Error = GenError;
            fn try_from(strategy: StrategyDto) -> GenResult<Self> {
                let mut dag: DiGraph<String, String> = DiGraph::new();
                let mut node_lookup = HashMap::new();

                // add nodes
                for calc in strategy.calcs() {
                    // println!("{}", calc.name());
                    let index = dag.add_node(calc.name().to_string());
                    node_lookup.insert(calc.name(), index);
                }
                // add edges
                for calc in strategy.calcs() {
                    for op in calc.operands() {
                        if node_lookup.contains_key(op.value())
                            && op._type() == &OperandType::Reference
                        {
                            let operand = node_lookup.get(op.value()).expect("operand not found");
                            let calc = node_lookup.get(calc.name()).expect("calc not found");
                            dag.add_edge(*operand, *calc, String::new());
                        }
                    }
                }
                match is_cyclic_directed(&dag) {
                    true => Err(InvalidStrategyError::new(
                        strategy.name().to_string(),
                        String::from("cyclic"),
                    )),
                    false => match connected_components(&dag) {
                        0 => Err(InvalidStrategyError::new(
                            strategy.name().to_string(),
                            String::from("zero connected components found"),
                        )),
                        1 => Ok(dag),
                        _ => Err(InvalidStrategyError::new(
                            strategy.name().to_string(),
                            String::from("more than 1 connected component found"),
                        )),
                    },
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use std::collections::HashMap;
            use std::convert::TryInto;
            use std::fs::read_to_string;
            use std::path::Path;

            use petgraph::algo::toposort;
            use petgraph::prelude::*;

            use crate::bot::dag::{Dag, DiGraph};
            use crate::dto::strategy::{from_path, StrategyDto};
            use crate::errors::GenResult;

            fn strategy_fixture() -> StrategyDto {
                from_path(Path::new("strategy.yaml")).expect("unable to load strategy")
            }

            fn dag_fixture() -> GenResult<Dag> {
                Dag::new(strategy_fixture())
            }

            #[test]
            fn strategy_to_dag() -> GenResult<()> {
                let strategy = strategy_fixture();
                let dag = Dag::new(strategy)?;
                dag.save_dot_file();
                let dag_dto = dag.dag_dto;
                assert_eq!(dag_dto.node_count(), 5);
                assert_eq!(dag_dto.edge_count(), 6);
                let nodes = dag_dto
                    .node_indices()
                    .map(|i| dag_dto.node_weight(i).expect("node not found"))
                    .find(|d| d.as_str().eq("sma200"))
                    .expect("sma200 not found");
                assert_eq!(nodes, "sma200");
                Ok(())
            }

            #[test]
            fn traverse_dag_order() -> GenResult<()> {
                let dag: Dag = dag_fixture()?;
                let exe_order = dag.execution_order();

                let node_execution_order_lkup: HashMap<&String, usize> = (0..exe_order.len())
                    .into_iter()
                    .map(|position| (exe_order.get(position).unwrap(), position))
                    .collect();
                let order_constraints = &[
                    &["price", "sma50", "sma_diff", "sma_gap"],
                    &["price", "sma200", "sma_diff", "sma_gap"],
                ];
                for outer_idx in 0..order_constraints.len() {
                    let expected_order = order_constraints[outer_idx];
                    for inner_idx in 0..(expected_order.len() - 1) {
                        let a = expected_order[inner_idx];
                        let a_position = node_execution_order_lkup.get(&a.to_string()).unwrap();
                        let b = expected_order[inner_idx + 1];
                        let b_position = node_execution_order_lkup.get(&b.to_string()).unwrap();
                        // println!("{:?}", (a, b));
                        // println!("{:?}", (a_position, b_position));
                        assert!(a_position < b_position);
                    }
                }
                Ok(())
            }

            #[test]
            fn dag_to_dot_file() -> GenResult<()> {
                let strategy = strategy_fixture();
                let dag = Dag::new(strategy)?;
                dag.save_dot_file();
                let expected_output =
                    read_to_string("expected_output.dot").expect("expected_output.dot not found.");
                let output = read_to_string("output.dot").expect("output.dot not found.");
                assert_eq!(output, expected_output);
                Ok(())
            }

            #[test]
            fn dag_upstream() -> GenResult<()> {
                let strategy_dto = strategy_fixture();
                let dag = Dag::new(strategy_dto)?;
                let upstream = dag.upstream(&String::from("sma50"));
                assert_eq!(upstream, vec![String::from("price")]);

                let upstream = dag.upstream(&String::from("sma_gap"));
                assert!(upstream.contains(&String::from("sma_diff")));
                assert!(upstream.contains(&String::from("sma50")));

                // println!("{:?}", upstream);
                Ok(())
            }

            #[test]
            fn topo() {
                // dag = C -> B <- A
                let mut dag: DiGraph<String, String> = DiGraph::new();
                let b = dag.add_node(String::from("B"));
                let c = dag.add_node(String::from("C"));
                let a = dag.add_node(String::from("A"));
                dag.add_edge(a, b, String::new());
                dag.add_edge(c, b, String::new());
                let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

                let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
                let node = dag.node_weight(*leaf_node_idx).unwrap();
                assert_eq!(node, "A");

                let leaf_node_idx = sorted_node_ids.get(1).expect("unable to get leaf");
                let node = dag.node_weight(*leaf_node_idx).unwrap();
                assert_eq!(node, "C");
            }

            #[test]
            fn dfs_post_order() -> GenResult<()> {
                let strategy = strategy_fixture();
                let dag: DiGraph<String, String> = strategy.clone().try_into()?;
                let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

                let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
                let leaf_node = dag
                    .node_weight(*leaf_node_idx)
                    .expect("unable to find node");
                assert_eq!(leaf_node, "price");

                let mut dfs_post_order = DfsPostOrder::new(&dag, *leaf_node_idx);
                let root_node_id = dfs_post_order.next(&dag).unwrap();
                let root_node: &String =
                    dag.node_weight(root_node_id).expect("unable to find root");
                // println!("{:?}", root_node);
                assert_eq!(root_node, strategy.score().calc());
                Ok(())
            }

            #[test]
            fn bfs() -> GenResult<()> {
                let strategy = strategy_fixture();
                let dag: DiGraph<String, String> = strategy.clone().try_into()?;
                let sorted_node_ids = toposort(&dag, None).expect("unable to toposort");

                let leaf_node_idx = sorted_node_ids.get(0).expect("unable to get leaf");
                let leaf_node = dag
                    .node_weight(*leaf_node_idx)
                    .expect("unable to find node");
                assert_eq!(leaf_node, "price");

                let mut bfs = Bfs::new(&dag, *leaf_node_idx);

                let mut node: &String;
                loop {
                    let node_id = bfs.next(&dag).unwrap();
                    node = dag.node_weight(node_id).expect("unable to find root");
                    // println!("{:?}", node);
                    if node == strategy.score().calc() {
                        break;
                    }
                }
                assert_eq!(node, strategy.score().calc());
                Ok(())
            }
        }
    }
}

pub mod dto {
    // TODO trade DTOs for messages to external broker service
    pub mod trade {}

    pub mod strategy {
        use std::borrow::BorrowMut;
        use std::convert::TryFrom;
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;

        use serde::{Deserialize, Serialize};

        use crate::errors::{GenError, GenResult};
        use crate::time_series::DataPointValue;

        pub type TimeSeriesReference = String;
        pub type TimeSeriesName = String;

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        #[allow(non_camel_case_types)]
        pub enum Operation {
            QUERY,
            ADD,
            SUB,
            MUL,
            DIV,
            TS_ADD,
            TS_SUB,
            TS_MUL,
            TS_DIV,
            SMA,
        }

        const DYADIC_TIME_SERIES_OPERATIONS: &[Operation] = &[
            Operation::TS_ADD,
            Operation::TS_SUB,
            Operation::TS_MUL,
            Operation::TS_DIV,
        ];

        const DYADIC_SCALAR_OPERATIONS: &[Operation] = &[
            Operation::ADD,
            Operation::SUB,
            Operation::MUL,
            Operation::DIV,
        ];

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        pub enum OperandType {
            Text,
            Integer,
            Decimal,
            Reference,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        pub struct ScoreDto {
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
        pub struct OperandDto {
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
        pub struct CalculationDto {
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

        // TODO parameterized query: generalize market data retrieval
        pub struct QueryCalculationDto {
            name: String,
            field: String,
        }

        impl QueryCalculationDto {
            pub fn name(&self) -> &str {
                &self.name
            }
            pub fn field(&self) -> &str {
                &self.field
            }
        }

        impl TryFrom<CalculationDto> for QueryCalculationDto {
            type Error = GenError;
            fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
                if calculation_dto.operation != Operation::QUERY {
                    Err(GenError::from("Conversion into QueryCalculationDto failed")).into()
                } else {
                    let name: String = calculation_dto.name.clone();
                    let field: String = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "field")
                        .ok_or("Conversion into QueryCalculationDto failed: field is required")?
                        .value
                        .clone();
                    Ok(Self { name, field })
                }
            }
        }

        pub struct DyadicTsCalculationDto {
            name: String,
            left: TimeSeriesReference,
            right: TimeSeriesReference,
        }

        impl DyadicTsCalculationDto {
            pub fn name(&self) -> &str {
                &self.name
            }
            pub fn left(&self) -> &TimeSeriesReference {
                &self.left
            }
            pub fn right(&self) -> &TimeSeriesReference {
                &self.right
            }
        }

        impl TryFrom<CalculationDto> for DyadicTsCalculationDto {
            type Error = GenError;
            fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
                if !DYADIC_TIME_SERIES_OPERATIONS.contains(&calculation_dto.operation) {
                    Err(GenError::from(
                        "Conversion into DyadicTsCalculationDto failed",
                    ))
                    .into()
                } else {
                    let name: String = calculation_dto.name.clone();
                    let left: TimeSeriesReference = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "left")
                        .ok_or("Conversion into DyadicTsCalculationDto failed: left is required")?
                        .value
                        .clone();
                    let right: TimeSeriesReference = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "right")
                        .ok_or("Conversion into DyadicTsCalculationDto failed: right is required")?
                        .value
                        .clone();
                    Ok(Self { name, left, right })
                }
            }
        }

        pub struct DyadicScalarCalculationDto {
            name: String,
            time_series: TimeSeriesReference,
            scalar: DataPointValue,
        }

        impl DyadicScalarCalculationDto {
            pub fn name(&self) -> &str {
                &self.name
            }
            pub fn time_series(&self) -> &TimeSeriesReference {
                &self.time_series
            }
            pub fn scalar(&self) -> DataPointValue {
                self.scalar
            }
        }

        impl TryFrom<CalculationDto> for DyadicScalarCalculationDto {
            type Error = GenError;
            fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
                if !DYADIC_SCALAR_OPERATIONS.contains(&calculation_dto.operation) {
                    Err(GenError::from(
                        "Conversion into DyadicScalarCalculationDto failed",
                    ))
                    .into()
                } else {
                    let name: String = calculation_dto.name.clone();
                    let time_series: TimeSeriesReference = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "time_series")
                        .ok_or(
                            "Conversion into DyadicScalarCalculationDto failed: time_series is required",
                        )?
                        .value
                        .clone();
                    let scalar: DataPointValue = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "scalar")
                        .ok_or(
                            "Conversion into DyadicScalarCalculationDto failed: scalar is required",
                        )?
                        .value
                        .parse()?;
                    Ok(Self {
                        name,
                        time_series,
                        scalar,
                    })
                }
            }
        }

        pub struct SmaCalculationDto {
            name: String,
            window_size: usize,
            time_series: TimeSeriesReference,
        }

        impl SmaCalculationDto {
            pub fn name(&self) -> &str {
                &self.name
            }
            pub fn window_size(&self) -> usize {
                self.window_size
            }
            pub fn time_series(&self) -> &TimeSeriesReference {
                &self.time_series
            }
        }

        impl TryFrom<CalculationDto> for SmaCalculationDto {
            type Error = GenError;
            fn try_from(calculation_dto: CalculationDto) -> GenResult<Self> {
                if calculation_dto.operation != Operation::SMA {
                    Err(GenError::from("Conversion into SmaCalculationDto failed")).into()
                } else {
                    let name: String = calculation_dto.name.clone();
                    let window_size: usize = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "window_size")
                        .ok_or("Conversion into SmaCalculationDto failed: window_size is required")?
                        .value
                        .parse()?;
                    let time_series: String = calculation_dto
                        .operands
                        .iter()
                        .find(|o| o.name == "time_series")
                        .ok_or("Conversion into SmaCalculationDto failed: time_series is required")?
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

        #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
        pub struct StrategyDto {
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
                &calcs
                    .iter()
                    .find(|c| c.name == score.calc)
                    .expect("Invalid strategy, score calc not found");
                StrategyDto { name, score, calcs }
            }
        }

        pub fn from_path(file_path: &Path) -> Result<StrategyDto, serde_yaml::Error> {
            let mut strategy_file: File = File::open(file_path).expect("unable to open file");
            let mut strategy_yaml = String::new();
            strategy_file
                .read_to_string(strategy_yaml.borrow_mut())
                .expect("unable to read strategy file");
            serde_yaml::from_str(&strategy_yaml)
        }

        #[cfg(test)]
        mod tests {
            use std::convert::TryInto;
            use std::env::current_dir;
            use std::path::Path;

            use crate::dto::strategy::*;
            use crate::errors::GenResult;

            pub fn get_strategy() -> StrategyDto {
                StrategyDto {
                    name: String::from("Example Strategy Document"),
                    score: ScoreDto {
                        calc: String::from("sma_gap"),
                    },
                    calcs: vec![
                        CalculationDto {
                            name: String::from("sma_gap"),
                            operation: Operation::TS_DIV,
                            operands: vec![
                                OperandDto {
                                    name: String::from("left"),
                                    _type: OperandType::Reference,
                                    value: String::from("sma_diff"),
                                },
                                OperandDto {
                                    name: String::from("right"),
                                    _type: OperandType::Reference,
                                    value: String::from("sma50"),
                                },
                            ],
                        },
                        CalculationDto {
                            name: String::from("sma_diff"),
                            operation: Operation::TS_SUB,
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
    operation: TS_DIV
    operands:
      - name: left
        type: Reference
        value: sma_diff
      - name: right
        type: Reference
        value: sma50
  - name: sma_diff
    operation: TS_SUB
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
                assert_eq!(s.calcs[0].operation, Operation::TS_DIV);
                assert_eq!(s.calcs[0].operands[0].name, "left");
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
operation: TS_DIV
operands:
  - name: left
    type: Reference
    value: foo
  - name: right
    type: Reference
    value: bar"#;
                let calc_dto: CalculationDto = serde_yaml::from_str(x)?;
                let sma: DyadicTsCalculationDto = calc_dto.try_into()?;
                assert_eq!(sma.name, "sma200");
                assert_eq!(sma.left, "foo");
                assert_eq!(sma.right, "bar");
                Ok(())
            }
        }
    }
}
