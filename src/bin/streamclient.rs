use std::{pin::Pin, task::Poll, time::Duration};

use rustgrpcdemo::{
    echopb::{EchoRequest, echo_client::EchoClient},
    now_formatted,
};
use tokio::time::Sleep;
use tokio_stream::Stream;

/// An example wrapper around `tokio::time::Sleep()` to help understand Futures.
struct SleepWrapper {
    wrapped: Pin<Box<Sleep>>,
}

impl SleepWrapper {
    fn new(duration: Duration) -> Self {
        Self {
            wrapped: Box::pin(tokio::time::sleep(duration)),
        }
    }
}

impl Future for SleepWrapper {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!(
            "{} SleepWrapper.poll() called; calling wrapped sleep",
            now_formatted()
        );
        let result = self.wrapped.as_mut().poll(cx);
        println!(
            "{}     SleepWrapper.poll() returning {result:?}",
            now_formatted(),
        );
        result
    }
}

/// A "by hand" implementation of an async stream that returns messages and sleeps between them.
struct RawRequestStream {
    num_messages_to_send: usize,
    message_sleep_duration: Duration,
    messages_sent: usize,
    sleep_future: Option<Pin<Box<Sleep>>>,
}

impl RawRequestStream {
    const fn new(num_messages_to_send: usize, message_sleep_duration: Duration) -> Self {
        Self {
            num_messages_to_send,
            message_sleep_duration,
            messages_sent: 0,
            sleep_future: None,
        }
    }
}

impl Stream for RawRequestStream {
    type Item = EchoRequest;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        println!("{} RawRequestStream: poll_next called ...", now_formatted());

        // if we have a sleep_future: poll it and return if it is Pending
        if let Some(sleep_future) = self.sleep_future.as_mut() {
            println!(
                "{}     RawRequestStream: sleep_future exists; polling it ...",
                now_formatted()
            );
            let pinned_sleep_future = sleep_future.as_mut();
            let result = pinned_sleep_future.poll(cx);
            println!(
                "{}     RawRequestStream: sleep_future returned {result:?}",
                now_formatted()
            );
            match result {
                Poll::Pending => return Poll::Pending,

                Poll::Ready(()) => {
                    // sleep done: drop the future and fall through
                    self.sleep_future = None;
                }
            }
        }
        assert!(self.sleep_future.is_none());

        if self.messages_sent == self.num_messages_to_send {
            println!(
                "{}     RawRequestStream: all messages sent: returning Ready(None)",
                now_formatted()
            );
            return std::task::Poll::Ready(None);
        }
        assert!(self.messages_sent < self.num_messages_to_send);

        let mut self_mut = self.as_mut();
        self_mut.messages_sent += 1;
        if self_mut.messages_sent < self_mut.num_messages_to_send {
            println!(
                "{}     RawRequestStream: sleeping between messages for {:?}",
                now_formatted(),
                self_mut.message_sleep_duration
            );
            // there are more messages to send: sleep after sending
            self_mut.sleep_future = Some(Box::pin(tokio::time::sleep(
                self_mut.message_sleep_duration,
            )));
        }

        let request = EchoRequest {
            input: format!("message {}", self_mut.messages_sent),
        };
        println!(
            "{}     RawRequestStream: returning Ready(Some(request.input={})",
            now_formatted(),
            request.input
        );
        std::task::Poll::Ready(Some(request))
    }
}

struct GetAllFuture {
    raw_stream: Pin<Box<RawRequestStream>>,
    result: Vec<EchoRequest>,
}

impl GetAllFuture {
    fn new(raw_stream: RawRequestStream) -> Self {
        Self {
            raw_stream: Box::pin(raw_stream),
            result: vec![],
        }
    }
}

impl Future for GetAllFuture {
    type Output = Vec<EchoRequest>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        loop {
            println!(
                "{} GetAllFuture.poll(): calling raw_stream.poll_next in loop ...",
                now_formatted()
            );

            let stream = self.raw_stream.as_mut();
            let result = stream.poll_next(cx);
            match result {
                Poll::Ready(None) => {
                    println!(
                        "{}     GetAllFuture.poll(): poll_next returned Ready(None); return Vec with {} messages",
                        now_formatted(),
                        self.result.len()
                    );
                    let result = std::mem::take(&mut self.result);
                    return Poll::Ready(result);
                }
                Poll::Ready(Some(request)) => {
                    println!(
                        "{}     GetAllFuture.poll(): poll_next returned Ready(Some(request)); adding and polling again",
                        now_formatted(),
                    );
                    self.result.push(request);
                }
                Poll::Pending => {
                    println!(
                        "{}     GetAllFuture.poll(): poll_next returned Pending; returning",
                        now_formatted(),
                    );
                    return Poll::Pending;
                }
            }
        }
    }
}

#[expect(clippy::significant_drop_tightening)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const GRPC_URL: &str = "http://[::1]:8001/";
    const NUM_MESSAGES: usize = 3;
    const MESSAGE_SLEEP: Duration = Duration::from_secs(1);
    const FUTURE_EXAMPLE_SLEEP: Duration = Duration::ZERO;
    // const FUTURE_EXAMPLE_SLEEP: Duration = Duration::from_millis(100);

    // example of a raw Future that wraps a tokio sleep
    println!(
        "{} SleepWrapper futures example: sleeping for {FUTURE_EXAMPLE_SLEEP:?} ...",
        now_formatted()
    );
    SleepWrapper::new(FUTURE_EXAMPLE_SLEEP).await;
    println!();

    // use our raw stream directly with await
    let result = GetAllFuture::new(RawRequestStream::new(2, Duration::ZERO)).await;
    println!(
        "{} GetAllFuture returned {} values",
        now_formatted(),
        result.len()
    );
    println!();

    println!(
        "{} stream client connecting to GRPC_URL={GRPC_URL} ...",
        now_formatted()
    );
    let mut client = EchoClient::connect(GRPC_URL).await?;

    println!(
        "{} starting stream using RawRequestStream ...",
        now_formatted()
    );
    // NOTE: the simplest thing seems to be to use this async-stream crate?
    // https://docs.rs/async-stream/latest/async_stream/
    // This is an experiment to implement it "by hand"
    let request_stream = RawRequestStream::new(NUM_MESSAGES, MESSAGE_SLEEP);

    let mut stream = client
        .echo_bi_dir(tonic::Request::new(request_stream))
        .await?
        .into_inner();
    let mut received_messages = 0;
    while let Some(response) = stream.message().await? {
        println!(
            "{} received response.output={}",
            now_formatted(),
            response.output
        );
        received_messages += 1;
    }
    println!(
        "{} stream complete received_messages={received_messages}",
        now_formatted()
    );

    Ok(())
}
