use std::marker::PhantomData;

use chrono::SecondsFormat;
use prost::Message;
use tonic::codec::BufferSettings;
use tonic::codec::Codec;
use tonic_prost::ProstCodec;

pub mod echopb {
    #![expect(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("echopb");
}

pub mod custom_codec_echopb {
    #![expect(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("custom_codec/echopb");
}

const PROTOBUF_TYPE_URL_PREFIX: &str = "type.googleapis.com/";

// Must be manually implemented since it does not yet have prost-build support.
impl prost::Name for echopb::Example1 {
    const NAME: &'static str = "Example1";
    const PACKAGE: &'static str = "echopb";

    fn type_url() -> String {
        // prost does not agree with Go's protobuf implementation:
        // "The default type URL for a given message type is
        // type.googleapis.com/_packagename_._messagename_."
        // https://protobuf.dev/programming-guides/proto3/#any
        format!("{PROTOBUF_TYPE_URL_PREFIX}{}", Self::full_name())
    }
}

// Must be manually implemented since it does not yet have prost-build support.
impl prost::Name for echopb::Example2 {
    const NAME: &'static str = "Example2";
    const PACKAGE: &'static str = "echopb";

    fn type_url() -> String {
        format!("{PROTOBUF_TYPE_URL_PREFIX}{}", Self::full_name())
    }
}

/// Returns the current `SystemTime` formatted for a log file.
#[must_use]
pub fn now_formatted() -> String {
    let now_utc = chrono::Utc::now();
    now_utc.to_rfc3339_opts(SecondsFormat::Micros, true)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CustomResponseCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Codec for CustomResponseCodec<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Encode = T;
    type Decode = U;

    type Encoder = <ProstCodec<T, U> as Codec>::Encoder;
    type Decoder = <ProstCodec<T, U> as Codec>::Decoder;

    fn encoder(&mut self) -> Self::Encoder {
        // Here, we will just customize the prost codec's internal buffer settings.
        // You can of course implement a complete Codec, Encoder, and Decoder if
        // you wish!
        panic!("encoder");
        // ProstCodec::<T, U>::raw_encoder(BufferSettings::new(512, 4096))
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProstCodec::<T, U>::raw_decoder(BufferSettings::new(512, 4096))
    }
}
