// cli library
use chrono::{DateTime, Utc};

use structopt::StructOpt;

use luckless::back_test::{BackTest, BackTestConfig};
use luckless::data::DataClient;
use luckless::dto::strategy::StrategyDto;
use luckless::errors::GenResult;
pub use luckless::query_client::{parse_date, parse_strategy_yaml, QueryClient};

use luckless::simulation::MockDataClient;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Execute given strategy to compute non-negative score of the given assets over the given time range."
)]
struct Opt {
    /// Use Query GRPC service instead of mock data
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    grpc: bool,
    /// first date in back test in RFC3339/ISO8601 format.
    #[structopt(short = "s", long = "start", parse(try_from_str = parse_date), default_value = "2011-12-01T00:00:00UTC")]
    start: DateTime<Utc>,
    /// first date in back test in RFC3339/ISO8601 format.
    #[structopt(short = "e", long = "end", parse(try_from_str = parse_date), default_value = "2012-01-01T00:00:00UTC")]
    end: DateTime<Utc>,
    /// path to strategy yaml file
    #[structopt(short = "f", long = "file", parse(try_from_str = parse_strategy_yaml), default_value = "./strategy.yaml")]
    pub(crate) strategy: StrategyDto,
    // TODO accept list of symbols
}

fn parse_args() -> Result<BackTestConfig, String> {
    let opt: Opt = Opt::from_args();
    // println!("strategy: {:?}", opt.strategy);
    // println!("start: {:?}", opt.start);
    // println!("end: {:?}", opt.end);
    if !(opt.start < opt.end) {
        return Err("!(start < end)".to_string());
    }
    let data_client: Box<dyn DataClient>;
    if opt.grpc {
        println!("Attempting GRPC");
        data_client = Box::new(QueryClient::new())
    } else {
        data_client = Box::new(MockDataClient::new());
    }
    Ok(BackTestConfig::new(opt.end, opt.strategy, data_client))
}

fn main() -> GenResult<()> {
    let parse_result = parse_args();
    if parse_result.is_err() {
        println!("{:?}", parse_result.err().expect("Unknown Error"))
    } else {
        let back_test_config: BackTestConfig = parse_result.unwrap();
        println!("back_test_config: {:?}\n", back_test_config);
        let back_test_result = back_test_config.compute_scores()?;
        println!("back_test_result: {:?}\n", back_test_result);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use luckless::back_test::{BackTest, BackTestConfig};
    use luckless::errors::GenResult;

    use luckless::simulation::MockDataClient;
    use luckless::time_series::TimeSeries1D;

    use crate::{parse_date, parse_strategy_yaml};

    #[test]
    fn main() -> GenResult<()> {
        let start_str = "2011-12-01T00:00:00UTC";
        let start = parse_date(start_str)?;
        let end_str = "2012-01-01T00:00:00UTC";
        let end = parse_date(end_str)?;
        let strategy_str = "./strategy.yaml";
        let strategy = parse_strategy_yaml(strategy_str)?;

        let _timestamps: Vec<DateTime<Utc>> = (0..(end - start).num_days())
            .map(|i| start + TimeSeries1D::index_unit() * i as i32)
            .collect();
        let data_client = Box::new(MockDataClient::new());
        // let data_client = Box::new(QueryClient::new());
        let back_test_config = BackTestConfig::new(end, strategy, data_client);
        println!("back_test_config: {:?}\n", back_test_config);
        let back_test_result = back_test_config.compute_scores()?;
        println!("back_test_result: {:?}\n", back_test_result);

        Ok(())
    }
}
