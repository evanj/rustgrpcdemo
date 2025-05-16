use prost::Message;
use prost::Name;
use rustgrpcdemo::{
    echopb::{EchoRequest, Example1, Example2, echo_client::EchoClient},
    now_formatted,
};

/// Returns the details from a gRPC grpc-status-details-bin response header.
/// If there is an error it return an empty Vec.
/// See: <https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md>
fn decode_details(details: &[u8]) -> Vec<prost_types::Any> {
    let Ok(details_status) = tonic_types::Status::decode(details) else {
        return vec![];
    };
    details_status.details
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const GRPC_URL: &str = "http://localhost:8001/";

    println!("{} connecting to GRPC_URL={GRPC_URL} ...", now_formatted());
    let mut client = EchoClient::connect(GRPC_URL).await?;

    let request = EchoRequest {
        input: "Hello, world!".to_string(),
    };
    match client.echo(request).await {
        Ok(response) => {
            let response = response.into_inner();
            println!(
                "{} received response.output={}",
                now_formatted(),
                response.output
            );
        }
        Err(grpc_status) => {
            let details = decode_details(grpc_status.details());
            println!(
                "{} code:{} {:?} details_len={} msg={}",
                now_formatted(),
                grpc_status.code() as i64,
                grpc_status.code(),
                details.len(),
                grpc_status.message()
            );
            for (i, detail) in details.iter().enumerate() {
                println!(
                    "  details i={i} type={} len={}",
                    detail.type_url,
                    detail.value.len()
                );
                let value_bytes: &[u8] = &detail.value;
                if detail.type_url == Example1::type_url() {
                    let example1 = Example1::decode(value_bytes).unwrap();
                    println!("    details i={i} {example1:?}");
                } else if detail.type_url == Example2::type_url() {
                    let example2 = Example2::decode(value_bytes).unwrap();
                    println!("    details i={i} {example2:?}");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_details() {
        // From a Go server
        const DETAILS_BYTES_HEX: &[u8] = &[
            0x08, 0x0D, 0x12, 0x14, 0x65, 0x72, 0x72, 0x6F, 0x72, 0x20, 0x77, 0x69, 0x74, 0x68,
            0x20, 0x32, 0x20, 0x64, 0x65, 0x74, 0x61, 0x69, 0x6C, 0x73, 0x1A, 0x29, 0x0A, 0x23,
            0x74, 0x79, 0x70, 0x65, 0x2E, 0x67, 0x6F, 0x6F, 0x67, 0x6C, 0x65, 0x61, 0x70, 0x69,
            0x73, 0x2E, 0x63, 0x6F, 0x6D, 0x2F, 0x65, 0x63, 0x68, 0x6F, 0x70, 0x62, 0x2E, 0x45,
            0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x31, 0x12, 0x02, 0x08, 0x63, 0x1A, 0x30, 0x0A,
            0x23, 0x74, 0x79, 0x70, 0x65, 0x2E, 0x67, 0x6F, 0x6F, 0x67, 0x6C, 0x65, 0x61, 0x70,
            0x69, 0x73, 0x2E, 0x63, 0x6F, 0x6D, 0x2F, 0x65, 0x63, 0x68, 0x6F, 0x70, 0x62, 0x2E,
            0x45, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x32, 0x12, 0x09, 0x09, 0x1F, 0x85, 0xEB,
            0x51, 0xB8, 0x1E, 0x09, 0x40,
        ];

        let result = decode_details(DETAILS_BYTES_HEX);
        assert_eq!(2, result.len());
        assert_eq!(result[0].type_url, Example1::type_url());
        assert_eq!(result[1].type_url, Example2::type_url());
    }
}
