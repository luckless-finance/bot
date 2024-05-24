# Bot
_A toolset for algorithmic stock trading_
> Part of the [Luckless platform](https://github.com/luckless-finance)

Bot
consumes stock market data to
determine the desired portfolio allocation of assets
based on a user defined stocking picking strategy.

## Use cases
- back testing a stock picking strategy
- managing a real world stock trading account (future)

## Develop

```bash
sudo apt-get install pkg-config libssl-dev gnuplot
rustup update stable
cargo update
make test
make release
./target/release/luckless -s "2010-01-01T00:00:00-00:00" -e "2012-01-01T00:00:00+00:00"
```

## Back Test Quick Start

> Note, `luckless` currently used mock data.

```bash
$ ./luckless --help
# luckless x.y.z
# Back test a financial stock picking strategy.
# 
# USAGE:
#     luckless [OPTIONS]
# 
# FLAGS:
#     -h, --help       Prints help information
#     -V, --version    Prints version information
# 
# OPTIONS:
#     -e, --end <end>          first date in back test in RFC3339/ISO8601 format [default: 2012-01-01T00:00:00+00:00]
#     -s, --start <start>      first date in back test in RFC3339/ISO8601 format [default: 2011-12-01T00:00:00+00:00]
#     -f, --file <strategy>    path to strategy yaml file [default: ./strategy.yaml]
```

1. Create a [strategy.yaml](./strategy.yaml) file
2. Choose a date range
3. Execute bot cli to generate performance report

## Roadmap

see [here](https://github.com/grahamcrowell/yafa-bot/projects/1)

## Luckless Component Apps

- [bot](https://github.com/luckless-finance/bot)
- [query](https://github.com/luckless-finance/query)
- [broker](https://github.com/luckless-finance/broker)

> [shared](https://github.com/luckless-finance/shared) [docs](https://github.com/luckless-finance/docs)
