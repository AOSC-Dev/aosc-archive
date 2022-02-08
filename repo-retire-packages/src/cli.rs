use clap::{App, AppSettings, Arg};

pub fn build_cli() -> App<'static> {
    App::new("repo-retire")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("retire")
                .about("Retire the packages")
                .arg(
                    Arg::new("inhibit")
                        .short('t')
                        .long("inhibit")
                        .takes_value(true)
                        .min_values(1)
                        .required(false)
                        .help("Wait and inhibit the specified systemd services"),
                )
                .arg(
                    Arg::new("out-of-tree")
                        .short('f')
                        .takes_value(false)
                        .long("out-of-tree")
                        .help("Also clean up the out-of-tree packages"),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .takes_value(true)
                        .required(true)
                        .help("Path to the p-vector config file"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .takes_value(true)
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::new("dry-run")
                        .short('d')
                        .long("dry-run")
                        .help("Just print what would be done"),
                ),
        )
        .subcommand(
            App::new("binning")
                .about("Slice the directory into fixed-sized chunks")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .takes_value(true)
                        .required(true)
                        .help("Path to the input directory"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .takes_value(true)
                        .required(true)
                        .help("Path to the output directory"),
                )
                .arg(
                    Arg::new("size")
                        .short('s')
                        .takes_value(true)
                        .required(true)
                        .help("Size of each bin"),
                ),
        )
}
