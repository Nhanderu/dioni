use super::get_cache_path;
use lazy_static::lazy_static;
use std::{env::args, process::exit};

const HELP_MSG: &str = "\
usage: dioni [<flags>]

Flags:
    -h, --help             Shows this message
    -v, --version          Shows the program's version
    -q, --quiet            Surpress any message to stdout
    --ignore-excess        Ignore songs that exceed the Spotify limit
    --add-excess-to-queue  Add songs that exceed the Spotify limit to the queue
    -f, --force-fetching   Force tracks fetching
    --force-auth           Force authentication
    --cache-path           Shows the cache path
";

lazy_static! {
    pub static ref ARGS: Args = parse_args();
}

pub struct Args {
    pub quiet: bool,
    pub ignore_excess: bool,
    pub add_excess_to_queue: bool,
    pub force_fetching: bool,
    pub force_auth: bool,
}

impl Args {
    fn validate(&self) {
        if self.quiet && !self.ignore_excess && !self.add_excess_to_queue {
            eprintln!("--quiet requires one of --ignore-excess or --add-excess-to-queue");
            exit(1);
        }
    }
}

fn parse_args() -> Args {
    let mut parsed_args: Args = Args {
        quiet: false,
        ignore_excess: false,
        add_excess_to_queue: false,
        force_fetching: false,
        force_auth: false,
    };
    for arg in args().skip(1) {
        match &arg[..] {
            "--help" => help(),
            "--version" => version(),
            "--quiet" => parsed_args.quiet = true,
            "--ignore-excess" => parsed_args.ignore_excess = true,
            "--add-excess-to-queue" => parsed_args.add_excess_to_queue = true,
            "--force-fetching" => parsed_args.force_fetching = true,
            "--force-auth" => parsed_args.force_auth = true,
            "--cache-path" => match get_cache_path() {
                Ok(cache_path) => {
                    println!("{}", cache_path.as_path().display());
                    exit(0);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    exit(1);
                }
            },
            _ if arg.starts_with("-") && !arg.starts_with("--") => {
                for flag in arg.chars().skip(1) {
                    match flag {
                        'h' => help(),
                        'v' => version(),
                        'q' => parsed_args.quiet = true,
                        'f' => parsed_args.force_fetching = true,
                        _ => {
                            eprintln!("Unknown flag \"-{}\".\n", flag);
                            println!("{}", HELP_MSG);
                            exit(1);
                        }
                    }
                }
                continue;
            }
            _ => {
                eprintln!("Unknown argument \"{}\".\n", arg);
                println!("{}", HELP_MSG);
                exit(1);
            }
        }
    }
    parsed_args.validate();
    parsed_args
}

fn help() {
    println!("{}", HELP_MSG);
    exit(0);
}

fn version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
    exit(0);
}
