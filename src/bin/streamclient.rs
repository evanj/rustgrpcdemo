use std::time::Duration;

use rustgrpcdemo::echopb::{echo_client::EchoClient, EchoRequest};
use tokio_stream::Stream;

struct RawRequestStream {
    num_messages_to_send: usize,
    messages_sent: usize,
}

impl RawRequestStream {
    const fn new(num_messages_to_send: usize) -> Self {
        Self {
            num_messages_to_send,
            messages_sent: 0,
            sleep_future: None,
        }
    }
}

impl Stream for RawRequestStream {
    type Item = EchoRequest;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut_self = self.pin

        // if let Some(sleep_future) = mut_self.sleep_future.as_mut() {

        //     match std::pin::Pin::new(&mut self.get_mut().sleep_future).poll(cx) {
        //         std::task::Poll::Pending => {
        //             println!("RawRequestStream poll_next: sleep_future pending");
        //             return std::task::Poll::Pending;
        //         }
        //         std::task::Poll::Ready(()) => {
        //             println!("RawRequestStream poll_next: sleep_future ready");
        //             self.get_mut().sleep_future = None;
        //         }
        //     }
        // }

        if self.messages_sent == self.num_messages_to_send {
            println!("RawRequestStream poll_next: messages_sent == num_messages_to_send stream terminated: returning Ready(None)");
            return std::task::Poll::Ready(None);
        }
        assert!(self.messages_sent < self.num_messages_to_send);

        mut_self.messages_sent += 1;

        let request = EchoRequest {
            input: format!("message {}", mut_self.messages_sent),
        };
        println!(
            "RawRequestStream poll_next: returning Ready(Some(request.input={})",
            request.input
        );
        std::task::Poll::Ready(Some(request))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const GRPC_URL: &str = "http://localhost:8001/";
    const NUM_MESSAGES: usize = 3;
    const MESSAGE_SLEEP: Duration = Duration::from_secs(1);

    println!("stream client connecting to GRPC_URL={GRPC_URL} ...");

    let mut client = EchoClient::connect(GRPC_URL).await?;

    println!("starting stream ...");
    // NOTE: the simplest thing seems to be to use this async-stream crate?
    // https://docs.rs/async-stream/latest/async_stream/
    // This is an experiment to implement it "by hand"
    let request_stream = RawRequestStream::new(NUM_MESSAGES);

    // tokio::spawn(async move {
    //     for i in 0..NUM_MESSAGES {
    //         let request = EchoRequest {
    //             input: format!("message {i}"),
    //         };
    //         println!("sending request.input={}", request.input);
    //         println!("todo!");
    //         // let send_result = request_stream_sender.send(Ok(request)).await;
    //         // if let Err(send_err) = send_result {
    //         //     eprintln!("request_stream_sender.send() failed: {send_err}");
    //         //     break;
    //         // }
    //         tokio::time::sleep(MESSAGE_SLEEP).await;
    //     }
    // });

    let mut stream = client
        .echo_bi_dir(tonic::Request::new(request_stream))
        .await?
        .into_inner();
    let mut received_messages = 0;
    while let Some(response) = stream.message().await? {
        println!("received response.output={}", response.output);
        received_messages += 1;
    }
    println!("stream complete received_messages={received_messages}");

    Ok(())
}
