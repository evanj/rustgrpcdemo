use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const EMPTY_PATH_SLICE: &[&Path] = &[];

    dlprotoc::download_protoc()?;
    tonic_build::compile_protos("proto/echo.proto")?;

    // make a copy of the protos with a custom codec
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let custom_codec_dir = out_dir.join("custom_codec");
    // use create_dir_all to ignore "directory exists" errors
    std::fs::create_dir_all(&custom_codec_dir)?;
    tonic_build::configure()
        .out_dir(custom_codec_dir)
        .codec_path("crate::CustomResponseCodec")
        .compile_protos(&["proto/echo.proto"], EMPTY_PATH_SLICE)?;
    Ok(())
}

// TODO: define a raw buffer output type
// Manually define the json.helloworld.Greeter service which used a custom JsonCodec to use json
// serialization instead of protobuf for sending messages on the wire.
// This will result in generated client and server code which relies on its request, response and
// codec types being defined in a module `crate::common`.
//
// See the client/server examples defined in `src/json-codec` for more information.
// fn build_json_codec_service() {
//     let greeter_service = tonic_build::manual::Service::builder()
//         .name("Greeter")
//         .package("json.helloworld")
//         .method(
//             tonic_build::manual::Method::builder()
//                 .name("say_hello")
//                 .route_name("SayHello")
//                 .input_type("crate::common::HelloRequest")
//                 .output_type("crate::common::HelloResponse")
//                 .codec_path("crate::common::JsonCodec")
//                 .build(),
//         )
//         .build();

//     tonic_build::manual::Builder::new().compile(&[greeter_service]);
// }
