use std::net::SocketAddr;
use std::pin::Pin;

use rustgrpcdemo::echopb::echo_server::Echo;
use rustgrpcdemo::echopb::echo_server::EchoServer;
use rustgrpcdemo::echopb::EchoRequest;
use rustgrpcdemo::echopb::EchoResponse;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

#[derive(Debug)]
struct EchoService {}

impl EchoService {
    const fn new() -> Self {
        Self {}
    }
}

impl EchoService {}

#[tonic::async_trait]
impl Echo for EchoService {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        println!("echo request.msg={:?}", request.get_ref());
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
            println!("wtf here?");
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
            println!("wtf here 2?");
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
        println!("echo_bi_dir echoing request.input={:?}", request.input);
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
    println!("echo_bi_dir request stream ended; TODO: send bonus message");
    Ok(())
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
    Server::builder()
        .add_service(EchoServer::new(echo_service))
        .serve(listen_addr)
        .await?;

    Ok(())
}
