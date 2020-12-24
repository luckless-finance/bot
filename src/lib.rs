#![allow(dead_code)]
#![allow(unused_imports)]
#[cfg(test)]
#[macro_use]
extern crate approx;

use std::env::current_dir;
use std::path::Path;

use crate::dag::to_dag;
use crate::strategy::{from_path, StrategyDto};

pub mod bot;
pub mod dag;
pub mod data;
pub mod simulation;
pub mod strategy;
pub mod time_series;

fn load_strategy() -> StrategyDto {
    let strategy_path = current_dir()
        .expect("unable to get working directory")
        .join(Path::new("strategy.yaml"));
    from_path(strategy_path.as_path()).expect("unable to load from file")
}

fn demo_strategy() {
    println!(
        "current working directory: {}",
        current_dir()
            .expect("unable to get working directory")
            .to_str()
            .expect("unable to convert to str")
    );
    let bot = to_dag(&load_strategy()).expect("unable to convert to bot");
    println!("{:?}", bot)
}

// sudo apt-get install -y gnuplot
// fn demo_gnuplot() {
//     let mut fg = Figure::new();
//     fg.axes2d()
//         .set_title("A plot", &[])
//         .set_legend(Graph(0.5), Graph(0.9), &[], &[])
//         .set_x_label("x", &[])
//         .set_y_label("y^2", &[])
//         .lines(
//             &[-3., -2., -1., 0., 1., 2., 3.],
//             &[9., 4., 1., 0., 1., 4., 9.],
//             &[Caption("Parabola")],
//         );
//     fg.show().unwrap();
// }
//
// fn main() {
//     demo_strategy();
//     demo_gnuplot();
// }
