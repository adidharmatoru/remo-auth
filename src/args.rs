use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Listening address
    /// ./file --address 0.0.0.0:8080
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    pub(crate) address: String,
}
