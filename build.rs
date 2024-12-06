fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlprotoc::download_protoc()?;
    tonic_build::compile_protos("proto/echo.proto")?;
    Ok(())
}
