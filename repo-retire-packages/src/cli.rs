use clap::{crate_version, App, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("repo-retire")
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("retire")
                .about("Retire the packages")
                .arg(
                    Arg::with_name("inhibit")
                        .short("t")
                        .long("inhibit")
                        .takes_value(true)
                        .min_values(1)
                        .required(false)
                        .help("Wait and inhibit the specified systemd services"),
                )
                .arg(
                    Arg::with_name("out-of-tree")
                        .short("f")
                        .takes_value(false)
                        .long("out-of-tree")
                        .help("Also clean up the out-of-tree packages"),
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the p-vector config file"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::with_name("dry-run")
                        .short("d")
                        .long("dry-run")
                        .help("Just print what would be done"),
                ),
        )
        .subcommand(
            SubCommand::with_name("binning")
                .about("Slice the directory into fixed-sized chunks")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the input directory"),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::with_name("size")
                        .short("s")
                        .takes_value(true)
                        .required(true)
                        .help("Size of each bin"),
                ),
        )
}
