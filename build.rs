extern crate dotenv;

use std::env;
use std::path::PathBuf;

use dotenv::dotenv;

const DEFAULT_PROTO: &str = "proto";

fn main() {
    dotenv().ok();
    let var = "PROTO_PATH";
    let proto_root = match env::var_os(var) {
        Some(val) => {
            println!("{}: {:?}", var, val);
            PathBuf::from(val)
        }
        None => {
            println!("{} is not defined in the environment.", var);
            PathBuf::from(DEFAULT_PROTO)
        }
    };
    let proto_path = proto_root.join("query.proto");
    println!("generating code from: {:?}", proto_path);
    protoc_rust_grpc::Codegen::new()
        .out_dir("src")
        .include("proto")
        .inputs(&[proto_path.to_str().expect("query.proto")])
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
