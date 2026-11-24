#[tokio::main]
async fn main() -> std::io::Result<()> {
  taptime_client::backend::run_daemon().await
}
