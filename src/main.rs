// use std::env::current_dir;
// use std::path::Path;
//
// use crate::dag::to_dag;
// use crate::dto::{from_path, Strategy};

// mod dag;
// mod data;
// mod dto;
// mod engine;
// mod foo;
// mod dag_flow;
//
// fn load_strategy() -> Strategy {
//     let strategy_path = current_dir()
//         .expect("unable to get working directory")
//         .join(Path::new("strategy.yaml"));
//
//     from_path(strategy_path.as_path()).expect("unable to load from file")
// }
//
// fn demo_strategy() {
//     println!(
//         "current working directory: {}",
//         current_dir()
//             .expect("unable to get working directory")
//             .to_str()
//             .expect("unable to convert to str")
//     );
//     let dag = to_dag(&load_strategy()).expect("unable to convert to dag");
//     println!("{:?}", dag)
// }

mod dag_flow;

fn mirror_mirror() {
    // // let trump_trend = 0.262;
    // // let remaining_votes = 83_589.;
    // // let biden_total_votes = 3_336_887.;
    // // let trump_total_votes = 3_308_054.;
    // //
    // // let trump_trend = 0.258;
    // // let remaining_votes = 67_830.;
    // // let biden_total_votes = 3_358_920.;
    // // let trump_total_votes = 3_315_726.;
    //
    // // let trump_trend = 0.254;
    // // let remaining_votes = 62_746.;
    // // let biden_total_votes = 3_361_700.;
    // // let trump_total_votes = 3_316_043.;
    //
    // // let trump_trend = 0.292;
    // // let remaining_votes = 57_671.;
    // // let biden_total_votes = 3_364_279.;
    // // let trump_total_votes = 3_318_876.;
    // // https://alex.github.io/nyt-2020-election-scraper/battleground-state-changes.html
    // let trump_trend = 0.419;
    // let remaining_votes = 34_286.;
    // let biden_total_votes = 3_379_367.;
    // let trump_total_votes = 3_329_152.;
    //
    // let biden_trend = (1. - trump_trend);
    // let biden_remaining_votes = biden_trend * remaining_votes;
    // let total_biden_votes = biden_total_votes + biden_remaining_votes;
    //
    // let trump_remaining_votes = trump_trend * remaining_votes;
    // let total_trump_votes = trump_total_votes + trump_remaining_votes;
    //
    // let biden_percentage = total_biden_votes / (total_biden_votes + total_trump_votes);
    // let trump_percentage = total_trump_votes / (total_biden_votes + total_trump_votes);
    //
    // let differential = biden_percentage - trump_percentage;
    //
    // if differential < 0.01 {
    //     println!("{} uncle russ will owe graham $5", differential)
    // } else {
    //     println!("{} graham will owe uncle russ $5", differential)
    // }

    let b = 3_412_806.;
    let t = 3_349_801.;
    let r = 42_766.;

    let x = (b + r) / (b + t + r);
    let y = (t) / (b + t + r);
    println!("{}", x);
    println!("{}", y);
    println!("{}", y - x);
    
}
fn main() {
    mirror_mirror()
}
