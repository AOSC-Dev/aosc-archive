use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("repo-retire")
        .arg_required_else_help(true)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("retire")
                .about("Retire the packages")
                .arg(
                    Arg::new("inhibit")
                        .short('t')
                        .long("inhibit")
                        .num_args(1..)
                        .required(false)
                        .help("Wait and inhibit the specified systemd services"),
                )
                .arg(
                    Arg::new("out-of-tree")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .long("out-of-tree")
                        .help("Also clean up the out-of-tree packages"),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .required(true)
                        .help("Path to the p-vector config file"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::new("dry-run")
                        .short('d')
                        .action(ArgAction::SetTrue)
                        .long("dry-run")
                        .help("Just print what would be done"),
                ),
        )
        .subcommand(
            Command::new("binning")
                .about("Slice the directory into fixed-sized chunks")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .required(true)
                        .help("Path to the input directory"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::new("size")
                        .short('s')
                        .required(true)
                        .help("Size of each bin"),
                ),
        )
}
