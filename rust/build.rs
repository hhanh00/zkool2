fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=NEAR_INTENTS_API_KEY");
    // tonic_prost_build::configure()
    //     .out_dir("src/")
    //     .compile_protos(&["../protos/service.proto"], &["../protos/"])?;
    // std::fs::rename(
    //     "src/cash.z.wallet.sdk.rpc.rs",
    //     "src/lwd.rs",
    // )?;
    Ok(())
}
