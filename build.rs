fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/orderbook/order-book-merger.proto")?;
    Ok(())
}