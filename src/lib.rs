use chrono::SecondsFormat;

pub mod echopb {
    #![expect(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("echopb");
}

/// Returns the current `SystemTime` formatted for a log file.
#[must_use]
pub fn now_formatted() -> String {
    let now_utc = chrono::Utc::now();
    now_utc.to_rfc3339_opts(SecondsFormat::Micros, true)
}
