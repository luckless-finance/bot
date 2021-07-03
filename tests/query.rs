#[cfg(test)]
mod tests {
    use futures::executor;
    use grpc::{ClientConf, ClientStubExt};
    use luckless::query_demo::query_server;
    use luckless::query_grpc::MarketDataClient;

    const DEFAULT_PORT: u16 = 50052;
    const HOST: &str = "localhost";

    #[test]
    fn query_single_data_point() {
        println!("gRPC client connecting to {}:{:?}", HOST, DEFAULT_PORT);
        let client =
            MarketDataClient::new_plain(HOST, DEFAULT_PORT, ClientConf::new()).expect("client");
        executor::block_on(async { query_server(&client).await });
    }
}
