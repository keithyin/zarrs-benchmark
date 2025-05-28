use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    BamRw(BamRwArgs),
}

#[derive(Args)]
pub struct BamRwArgs {


}
