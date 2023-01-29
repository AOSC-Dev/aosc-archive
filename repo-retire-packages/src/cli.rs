use clap::Parser;

#[derive(Parser)]
pub struct RetireArgs {
    /// Wait and inhibit the specified systemd services
    #[arg(short = 't', long)]
    pub inhibit: Vec<String>,

    /// Also clean up the out-of-tree packages
    #[arg(short = 'f', long, default_value_t = false)]
    pub out_of_tree: bool,

    /// Path to the p-vector config file
    #[arg(short = 'c', long)]
    pub config: String,

    /// Path to the output directory
    #[arg(short = 'o', long)]
    pub output: String,

    /// Just print what would be done
    #[arg(short = 'd', long = "dry-run", default_value_t = false)]
    pub dry_run: bool,

    /// Save the data to the SQLite database at this path
    #[arg(short = 'b', long)]
    pub database: String,
}

#[derive(Parser)]
pub struct BinningArgs {
    /// Path to the input directory
    #[arg(short = 'i', long)]
    input: String,
    /// Path to the output directory
    #[arg(short = 'o', long)]
    output: String,
    /// Size of each bin
    #[arg(short = 's', long)]
    size: String,
}

#[derive(Parser)]
#[command(author, version, about)]
pub enum Args {
    /// Retire the packages
    Retire(RetireArgs),
    /// Slice the directory into fixed-sized chunks
    Binning(BinningArgs),
}
