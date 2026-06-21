use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, clap::Parser)]
#[command(name = "taptime server")]
#[command(version, about, long_about = None)]
#[command(next_line_help = false)]
pub struct Args {
  #[arg(short, long, default_value = "127.0.0.1:50051")]
  pub address: SocketAddr,

  #[arg(short = 'L', long, default_value_t = tracing::Level::INFO)]
  pub log_level: tracing::Level,

  #[arg(long, env = "DATABASE_URL")]
  pub database_url: String,

  #[arg(long, env = "JWT_SECRET")]
  pub jwt_secret: String,

  #[arg(long, env = "ADMIN_PASSWORD_HASH")]
  pub admin_password_hash: Option<String>,

  #[arg(long, env = "ADMIN_TOKEN_TTL_SECONDS", default_value_t = 28_800)]
  pub admin_token_ttl_seconds: i64,

  #[arg(long, env = "TRUST_PROXY_HEADERS", default_value_t = false)]
  pub trust_proxy_headers: bool,
}

#[must_use]
pub fn parse() -> Args {
  Args::parse()
}
