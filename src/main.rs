use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;
use clap::Parser;
use prost::Message;
use prost_types::Any;
use rustgrpcdemo::echopb::EchoRequest;
use rustgrpcdemo::echopb::EchoResponse;
use rustgrpcdemo::echopb::echo_server::Echo;
use rustgrpcdemo::echopb::echo_server::EchoServer;
use rustgrpcdemo::echopb::{Example1, Example2};
use rustgrpcdemo::now_formatted;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tonic::transport::Server;

#[derive(Debug)]
struct EchoService {
    err_details: bool,
}

impl EchoService {
    const fn new(err_details: bool) -> Self {
        Self { err_details }
    }
}

impl EchoService {}

#[tonic::async_trait]
impl Echo for EchoService {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        println!("echo request.msg={:?}", request.get_ref());
        if self.err_details {
            let details1_any = Any::from_msg(&Example1 { int64_value: 99 }).unwrap();
            let example2 = Example2 {
                float64_value: 1.234,
            };
            let details2_any = Any::from_msg(&example2).unwrap();

            let status_pb = tonic_types::Status {
                code: tonic::Code::Internal as i32,
                message: "error with 2 details".to_string(),
                details: vec![details1_any, details2_any],
            };
            // encode the status and attach it
            let status_bytes = status_pb.encode_to_vec();
            let status = tonic::Status::with_details(
                tonic::Code::Internal,
                "error with 2 details".to_string(),
                Bytes::from(status_bytes),
            );

            return Err(status);
        }

        let response = EchoResponse {
            output: format!("echoed: {}", request.get_ref().input),
        };
        Ok(Response::new(response))
    }

    type EchoBiDirStream = Pin<
        Box<dyn tokio_stream::Stream<Item = Result<EchoResponse, tonic::Status>> + Send + 'static>,
    >;

    async fn echo_bi_dir(
        &self,
        request: tonic::Request<tonic::Streaming<EchoRequest>>,
    ) -> std::result::Result<tonic::Response<Self::EchoBiDirStream>, tonic::Status> {
        println!("echo_bi_dir: starting new echo_bi_dir stream ...");
        let request_stream = request.into_inner();
        let (response_stream_sender, response_stream_rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            let stream_result = do_echo_bi_dir(request_stream, &response_stream_sender).await;
            if let Err(stream_err) = stream_result {
                eprintln!("do_echo_bi_dir returned error; sending to caller: {stream_err}");
                let final_send_result = response_stream_sender
                    .send(Err(tonic::Status::internal(format!(
                        "do_echo_bi_dir returned error: {stream_err}"
                    ))))
                    .await;
                if let Err(send_err) = final_send_result {
                    eprintln!("echo_bi_dir failed sending error to caller; send error: {send_err}");
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(
            response_stream_rx,
        ))))
    }
}

async fn do_echo_bi_dir(
    request_stream: tonic::Streaming<EchoRequest>,
    response_stream_sender: &tokio::sync::mpsc::Sender<Result<EchoResponse, tonic::Status>>,
) -> Result<(), tonic::Status> {
    let mut request_stream = request_stream;
    while let Some(request) = request_stream.message().await? {
        println!(
            "{} echo_bi_dir received request.input={:?}",
            now_formatted(),
            request.input
        );

        // tokio::time::sleep(Duration::from_millis(500)).await;
        // println!("{} unblocked after sleeping", now_formatted(),);

        let response = EchoResponse {
            output: format!("echoed: {}", request.input),
        };
        response_stream_sender
            .send(Ok(response))
            .await
            .map_err(|err| {
                tonic::Status::internal(format!(
                    "do_echo_bi_dir: response_stream_sender.send() failed: {err}"
                ))
            })?;
    }
    let extra_message = EchoResponse {
        output: "extra message after sender closed abcdef".to_string(),
    };
    println!(
        "{} echo_bi_dir request stream ended; sending extra bonus message: {}",
        now_formatted(),
        extra_message.output
    );
    response_stream_sender
        .send(Ok(extra_message))
        .await
        .map_err(|err| {
            tonic::Status::internal(format!(
                "do_echo_bi_dir: response_stream_sender.send() failed: {err}"
            ))
        })?;
    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    /// Returns a gRPC error with details that are compatible with other gRPC implementations.
    #[clap(long, default_value_t = false)]
    err_details: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const LISTEN_ADDR: &str = "::1";
    const LISTEN_PORT: u16 = 8001;
    let listen_addr_parsed = LISTEN_ADDR.parse().unwrap();
    let listen_addr = SocketAddr::new(listen_addr_parsed, LISTEN_PORT);

    let args = Args::parse();

    // construct our server
    let echo_service = EchoService::new(args.err_details);

    println!(
        "listening on {LISTEN_ADDR}:{LISTEN_PORT} err_details={}...",
        args.err_details
    );

    // create the grpc wrappers and start listening
    Server::builder()
        .add_service(EchoServer::new(echo_service))
        .serve(listen_addr)
        .await?;

    Ok(())
}
