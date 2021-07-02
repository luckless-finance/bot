use crate::query::RangedRequest;
use crate::query_grpc::MarketDataClient;
use crate::time_series::TimeStamp;
use chrono::format::Numeric::Timestamp;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use futures::StreamExt;
use std::time::SystemTime;

fn build_request() -> RangedRequest {
    let mut request = RangedRequest::new();
    request.symbol = "hello from rust!".to_string();
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

pub async fn stream_query_server(client: &MarketDataClient) {
    println!("query server for stream");
    let resp = client.query_stream(grpc::RequestOptions::new(), build_request());
    let mut stream = resp.drop_metadata();
    while let Some(data_point_result) = stream.next().await {
        match data_point_result {
            Ok(data_point) => {
                println!("query server for stream 4");
                let timestamp = data_point.clone().timestamp.unwrap();
                let timestamp: TimeStamp =
                    Utc.timestamp(timestamp.seconds, timestamp.nanos.abs() as u32);
                println!(
                    "timestamp: '{}', double: {:?}\n",
                    timestamp.to_rfc3339(),
                    data_point.value
                );
            }
            Err(e) => {
                println!("error connecting query server: {:?}", e);
            }
        }
    }
}
