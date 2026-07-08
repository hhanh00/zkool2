fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .out_dir("src/")
        .compile_protos(&["../protos/service.proto"], &["../protos/"])?;
    // tonic_prost_build names the output after the proto package
    // (cash.z.wallet.sdk.rpc). Rename to the module name lib.rs expects.
    std::fs::rename(
        "src/cash.z.wallet.sdk.rpc.rs",
        "src/lwd.rs",
    )?;
    Ok(())
}
