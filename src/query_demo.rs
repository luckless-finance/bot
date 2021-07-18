use std::time::SystemTime;

use chrono::format::Numeric::Timestamp;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use futures::StreamExt;
use protobuf::{Clear, SingularPtrField};

use crate::query::RangedRequest;
use crate::query_grpc::MarketDataClient;
use crate::time_series::TimeStamp;

pub fn from_proto(pb: protobuf::well_known_types::Timestamp) -> TimeStamp {
    return Utc::timestamp(&Utc, pb.seconds, pb.nanos.abs() as u32);
}

pub fn to_proto(ts: TimeStamp) -> protobuf::well_known_types::Timestamp {
    let ns = ts.timestamp_nanos() % 1_000_000_000i64;
    let s = (ts.timestamp_nanos() - ns) / 1_000_000_000i64;
    let mut pb = protobuf::well_known_types::Timestamp::new();
    pb.clear();
    pb.set_seconds(s);
    pb.set_nanos(ns as i32);
    pb
}

fn build_request() -> RangedRequest {
    let mut request = RangedRequest::new();
    let now_pb = to_proto(Utc::now());

    request.symbol = "RUST".to_string();
    request.series = "CLOSE".to_string();
    request.first = SingularPtrField::some(now_pb.clone());
    request.last = SingularPtrField::some(now_pb);
    request
}

pub async fn query_server(client: &MarketDataClient) {
    println!("query server non-stream");
    let result = client
        .query(grpc::RequestOptions::new(), build_request())
        .await;
    if result.is_err() {
        println!("error connecting query server: {:?}", result.err().unwrap());
    } else {
        println!("connected to query server");
        let (_meta, resp) = result.unwrap();
        for data_point in resp.await.unwrap().0.data.iter() {
            let timestamp = data_point.clone().timestamp.unwrap();
            let timestamp: TimeStamp =
                Utc.timestamp(timestamp.seconds, timestamp.nanos.abs() as u32);
            println!(
                "timestamp: '{}', double: {:?}\n",
                timestamp.to_rfc3339(),
                data_point.value
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};

    use crate::query_demo::{from_proto, to_proto};

    #[test]
    fn to_then_from() {
        let now: DateTime<Utc> = Utc::now();
        let proto_now = to_proto(now);
        let now_from_proto = from_proto(proto_now);

        assert_eq!(now.to_rfc3339(), now_from_proto.to_rfc3339());
        assert_eq!(now.timestamp(), now_from_proto.timestamp());
        assert_eq!(now.timestamp_nanos(), now_from_proto.timestamp_nanos());
    }

    #[test]
    fn from_then_to() {
        let now_pb = protobuf::well_known_types::Timestamp::new();
        let now: DateTime<Utc> = from_proto(now_pb.clone());
        let now_to_proto = to_proto(now);

        assert_eq!(now_pb.get_seconds(), now_to_proto.get_seconds());
        assert_eq!(now_pb.get_nanos(), now_to_proto.get_nanos());
    }
}
