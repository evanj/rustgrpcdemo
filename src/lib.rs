use chrono::SecondsFormat;

pub mod echopb {
    #![expect(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("echopb");
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
