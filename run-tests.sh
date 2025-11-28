clear; 
echo "clear; RUST_LOG='tungstenite=debug,tokio_tungstenite=debug,ttrs=trace' cargo test..."
RUST_LOG="tungstenite=debug,tokio_tungstenite=debug,ttrs=trace" cargo test
