pub mod echopb {
    #![expect(clippy::pedantic, clippy::nursery)]
    tonic::include_proto!("echopb");
}
