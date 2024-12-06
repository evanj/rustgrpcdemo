pub mod echopb {
    #![allow(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("echopb");
}

use std::net::SocketAddr;

use echopb::echo_server::Echo;
use echopb::echo_server::EchoServer;
use echopb::EchoRequest;
use echopb::EchoResponse;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

#[derive(Debug)]
struct EchoService;

impl EchoService {
    const fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl Echo for EchoService {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        println!("echo request.msg={:?}", request.get_ref());
        let response = EchoResponse {
            output: format!("echoed: {}", request.get_ref().input),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const LISTEN_ADDR: &str = "::1";
    const LISTEN_PORT: u16 = 8001;
    let listen_addr_parsed = LISTEN_ADDR.parse().unwrap();
    let listen_addr = SocketAddr::new(listen_addr_parsed, LISTEN_PORT);

    // construct our server
    let echo_service = EchoService::new();

    println!("listening on {LISTEN_ADDR}:{LISTEN_PORT} ...");

    // create the grpc wrappers and start listening
    let echo_server = EchoServer::new(echo_service);
    Server::builder()
        .add_service(echo_server)
        .serve(listen_addr)
        .await?;

    Ok(())
}
