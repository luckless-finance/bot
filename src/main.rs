// cli library
use chrono::{DateTime, Utc};
use luckless::back_test::{BackTest, BackTestConfig};
use luckless::dto::strategy::{from_path, StrategyDto};
use luckless::errors::GenResult;
use luckless::simulation::MockDataClient;
use luckless::time_series::TimeSeries1D;
use std::path::PathBuf;
use structopt::StructOpt;

#[allow(dead_code)]
fn parse_strategy_path(arg: &str) -> Result<PathBuf, String> {
    let strategy_path = PathBuf::from(arg);
    if !strategy_path.exists() {
        return Err(format!(
            "File does not exist. Expected yaml strategy file.  Got: {:?}",
            arg
        ));
    }
    if !strategy_path.is_file() {
        return Err(format!(
            "Not a file.  Expected yaml strategy file.  Got: {:?}",
            arg
        ));
    }
    match strategy_path.canonicalize() {
        Ok(absolute_path) => Ok(absolute_path),
        Err(error) => Err(error.to_string()),
    }
}

#[allow(dead_code)]
fn parse_strategy_yaml(arg: &str) -> Result<StrategyDto, String> {
    let strategy_path: PathBuf = parse_strategy_path(arg)?;
    match from_path(strategy_path.as_path()) {
        Ok(strategy_dto) => Ok(strategy_dto),
        Err(e) => Err(format!(
            "Unable to parse strategy yaml.  Got: {:?}\n{}",
            arg,
            e.to_string()
        )),
    }
}

#[allow(dead_code)]
fn parse_date(arg: &str) -> Result<DateTime<Utc>, String> {
    match DateTime::parse_from_rfc3339(arg) {
        Ok(start) => Ok(DateTime::from(start)),
        Err(e) => Err(format!(
            "Unable to parse start.  Expected RFC3339/ISO8601.  Got: {:?}\n{}",
            arg,
            e.to_string()
        )),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Back test a financial stock picking strategy.")]
struct Opt {
    /// first date in back test in RFC3339/ISO8601 format.
    #[structopt(short = "s", long = "start", parse(try_from_str = parse_date), default_value = "2011-12-01T00:00:00UTC")]
    start: DateTime<Utc>,

    /// first date in back test in RFC3339/ISO8601 format.
    #[structopt(short = "e", long = "end", parse(try_from_str = parse_date), default_value = "2012-01-01T00:00:00UTC")]
    end: DateTime<Utc>,
    /// path to strategy yaml file
    #[structopt(short = "f", long = "file", parse(try_from_str = parse_strategy_yaml), default_value = "./strategy.yaml")]
    strategy: StrategyDto,
}

fn parse_args() -> Result<BackTestConfig, String> {
    let opt: Opt = Opt::from_args();
    // println!("strategy: {:?}", opt.strategy);
    // println!("start: {:?}", opt.start);
    // println!("end: {:?}", opt.end);
    if !(opt.start < opt.end) {
        return Err("!(start < end)".to_string());
    }
    let timestamps: Vec<DateTime<Utc>> = (0..(opt.end - opt.start).num_days())
        .map(|i| opt.start + TimeSeries1D::index_unit() * i as i32)
        .collect();
    let data_client = Box::new(MockDataClient::new());
    Ok(BackTestConfig::new(timestamps, opt.strategy, data_client))
}

fn main() -> GenResult<()> {
    let parse_result = parse_args();
    if parse_result.is_err() {
        println!("{:?}", parse_result.err().expect("Unknown Error"))
    } else {
        let back_test_config: BackTestConfig = parse_result.unwrap();
        println!("back_test_config: {:?}\n", back_test_config);
        let back_test_result = back_test_config.compute_result(None)?;
        println!("back_test_result: {:?}\n", back_test_result);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{parse_date, parse_strategy_yaml};
    use chrono::{DateTime, Utc};
    use luckless::back_test::{BackTest, BackTestConfig};
    use luckless::errors::GenResult;
    use luckless::simulation::MockDataClient;
    use luckless::time_series::TimeSeries1D;

    #[test]
    fn main() -> GenResult<()> {
        let start_str = "2011-12-01T00:00:00UTC";
        let start = parse_date(start_str)?;
        let end_str = "2012-01-01T00:00:00UTC";
        let end = parse_date(end_str)?;
        let strategy_str = "./strategy.yaml";
        let strategy = parse_strategy_yaml(strategy_str)?;

        let timestamps: Vec<DateTime<Utc>> = (0..(end - start).num_days())
            .map(|i| start + TimeSeries1D::index_unit() * i as i32)
            .collect();
        let data_client = Box::new(MockDataClient::new());
        let back_test_config = BackTestConfig::new(timestamps, strategy, data_client);
        let allocations = back_test_config.compute_allocations()?;
        let back_test_result = back_test_config.compute_result(Some(allocations))?;
        Ok(())
    }
}
