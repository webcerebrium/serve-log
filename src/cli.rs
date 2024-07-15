use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "server-log",
    about = "Server that logs all requests details into stdout with headers and payload"
)]
pub struct Cli {
    /// Bind address
    #[clap(long, env = "BIND_ADDR", default_value = "0.0.0.0:5000")]
    pub bind_addr: String,
    /// Path to web root (storage of additional files)
    #[clap(long, env = "WEB_ROOT", default_value = "./")]
    pub web_root: String,
    /// Logging level
    #[clap(long, env = "RUST_LOG", default_value = "INFO")]
    pub log_level: String,
}
