// build script for cargo 
// here we configure tonic

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/hello.proto")?;
    tonic_build::compile_protos("proto/image_encoding.proto")?;
    Ok(())
}