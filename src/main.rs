// cli library
use chrono::{DateTime, Utc};
use structopt::StructOpt;

use luckless::bot::asset_score::RunnableStrategy;
use luckless::data::DataClient;
use luckless::dto::strategy::StrategyDto;
use luckless::errors::{CliArgError, GenResult};
use luckless::mock_client::MockDataClient;
pub use luckless::query_client::{parse_date, parse_strategy_yaml, QueryClient};
use luckless::time_series::TimeStamp;

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

fn parse_args() -> GenResult<(RunnableStrategy, TimeStamp)> {
    let opt: Opt = Opt::from_args();
    // println!("strategy: {:?}", opt.strategy);
    // println!("start: {:?}", opt.start);
    // println!("end: {:?}", opt.end);
    if !(opt.start < opt.end) {
        return Err(CliArgError::new("!(start < end)".to_string()));
    }
    let data_client: Box<dyn DataClient>;
    if opt.grpc {
        println!("Attempting GRPC");
        data_client = Box::new(QueryClient::new())
    } else {
        data_client = Box::new(MockDataClient::new());
    }
    Ok((RunnableStrategy::new(opt.strategy, data_client)?, opt.end))
}

fn main() -> GenResult<()> {
    let parse_result = parse_args();
    if parse_result.is_err() {
        println!("{:?}", parse_result.err().expect("Unknown Error"))
    } else {
        let (runnable_strategy, time_stamp) = parse_result.unwrap();
        println!("runnable_strategy: {:?}\n", runnable_strategy);
        let asset_scores = runnable_strategy.run_on_all_assets(time_stamp)?;
        println!("asset_scores: {:?}\n", asset_scores);
    }
    Ok(())
}
