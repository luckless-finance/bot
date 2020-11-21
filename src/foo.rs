// type TimeStamp = i64;
//
// trait Asset {}
//
// struct AssetImpl {}
//
// impl AssetImpl {
//     pub fn new() -> Box<Self> {
//         Box::from(AssetImpl {})
//     }
// }
//
// impl Asset for AssetImpl {}
//
// trait TimeSeries {
//     fn asset(&self) -> dyn Asset;
// }
//
// struct TimeSeriesImpl {
//     asset: dyn Asset
// }
//
// impl TimeSeriesImpl {
//     pub fn new(asset: Box<dyn Asset>) -> Box<Self> {
//         Box::from(TimeSeriesImpl { asset })
//     }
// }
//
// impl TimeSeries for TimeSeriesImpl {
//     fn asset(&self) -> &dyn Asset {
//         &self.asset
//     }
// }
//
// trait Query {}
//
// trait TimeSeriesQuery {
//     fn asset(&self) -> &dyn Asset;
// }
//
// struct TimeSeriesQueryImpl {
//     asset: Box<dyn Asset>
// }
//
// impl TimeSeriesQueryImpl {
//     pub fn new(asset: Box<dyn Asset>) -> Box<TimeSeriesQueryImpl> {
//         Box::from(TimeSeriesQueryImpl { asset })
//     }
// }
//
// impl TimeSeriesQuery for TimeSeriesQueryImpl {
//     fn asset(&self) -> &dyn Asset {}
// }
//
// trait DataSource {
//     fn query_time_series(&self, query: &dyn TimeSeriesQuery) -> Box<dyn TimeSeries>;
// }
//
// trait Score {}
//
// struct ScoreImpl {}
//
// impl ScoreImpl {
//     pub fn new() -> Box<Self> {
//         Box::from(ScoreImpl {})
//     }
// }
// impl Score for ScoreImpl {}
// trait Bot {
//     fn data_source(&self) -> &dyn DataSource;
//     fn compute_score(&self, asset: &dyn Asset, timestamp: TimeStamp) -> dyn Score;
// }
//
// trait Dag {
//     fn down_stream(&self, dag_node: &dyn DagNode) -> Vec<&dyn DagNode>;
//     fn up_stream(&self, dag_node: &dyn DagNode) -> Vec<&dyn DagNode>;
//     fn leaves(&self) -> Vec<&dyn DagNode>;
// }
//
// trait DagNodeOutput {}
//
// impl DagNodeOutput for dyn TimeSeries {}
//
// trait DagNodeInput {}
//
// struct QueryImpl {}
//
// impl DagNodeInput for QueryImpl {}
//
// impl DagNode for QueryImpl {
//     fn execute(&self, input: &dyn DagNodeInput) -> Box<dyn DagNodeOutput> {
//         TimeSeriesImpl::new(Box::from(AssetImpl::new()))
//     }
// }
//
// impl DagNodeOutput for dyn DagNodeInput {}
//
// trait DagNode {
//     fn execute(&self, input: &dyn DagNodeInput) -> dyn DagNodeOutput;
// }
//
// struct BotImpl {
//     data_source: Box<dyn DataSource>,
//     dag: dyn Dag,
// }
//
// impl Bot for BotImpl {
//     fn data_source(&self) -> &dyn DataSource {
//         &self.data_source
//     }
//
//     fn compute_score(&self, asset: &dyn Asset, timestamp: i64) -> Box<dyn Score> {
//         let upstream: Vec<&dyn DagNode> = self.dag.leaves().iter().flat_map(|query| self.dag.down_stream(query)).collect();
//         let query_results = self.dag.leaves().iter().map(|&query| query.execute()).collect();
//
//         let query = TimeSeriesQueryImpl::new(AssetImpl::new());
//         // Box::from(self.data_source.query_time_series(&query));
//         ScoreImpl::new()
//     }
// }
//
// trait BackTest {
//     fn bot(&self) -> &dyn Bot;
//     fn data_source(&self) -> &dyn DataSource;
//     fn start_date(&self) -> TimeStamp;
//     fn end_date(&self) -> Option<TimeStamp>;
// }
//
// #[test]
// fn mirror_mirror() {
//     // let trump_trend = 0.262;
//     // let remaining_votes = 83_589.;
//     // let biden_total_votes = 3_336_887.;
//     // let trump_total_votes = 3_308_054.;
//     //
//     // let trump_trend = 0.258;
//     // let remaining_votes = 67_830.;
//     // let biden_total_votes = 3_358_920.;
//     // let trump_total_votes = 3_315_726.;
//
//     // let trump_trend = 0.254;
//     // let remaining_votes = 62_746.;
//     // let biden_total_votes = 3_361_700.;
//     // let trump_total_votes = 3_316_043.;
//
//     // let trump_trend = 0.292;
//     // let remaining_votes = 57_671.;
//     // let biden_total_votes = 3_364_279.;
//     // let trump_total_votes = 3_318_876.;
//     // https://alex.github.io/nyt-2020-election-scraper/battleground-state-changes.html
//     let trump_trend = 0.419;
//     let remaining_votes = 34_286.;
//     let biden_total_votes = 3_379_367.;
//     let trump_total_votes = 3_329_152.;
//
//     let biden_trend = (1. - trump_trend);
//     let biden_remaining_votes = biden_trend * remaining_votes;
//     let total_biden_votes = biden_total_votes + biden_remaining_votes;
//
//     let trump_remaining_votes = trump_trend * remaining_votes;
//     let total_trump_votes = trump_total_votes + trump_remaining_votes;
//
//     let biden_percentage = total_biden_votes / (total_biden_votes + total_trump_votes);
//     let trump_percentage = total_trump_votes / (total_biden_votes + total_trump_votes);
//
//     let differential = biden_percentage - trump_percentage;
//
//     if differential < 0.01 {
//         println!("{} uncle russ will owe graham $5", differential)
//     } else {
//         println!("{} graham will owe uncle russ $5", differential)
//     }
// }