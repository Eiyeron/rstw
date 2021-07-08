use arg::Args;
use arg::ParseError;
use std::env;

#[derive(Args)]
pub struct TracerArgs {
    #[arg(long = "width", default_value = "400")]
    pub width: usize,

    #[arg(long = "height", default_value = "300")]
    pub height: usize,

    #[arg(short = "d", long = "depth", default_value = "10")]
    pub depth: u16,

    #[arg(short = "s", long = "samples", default_value = "100")]
    pub samples: usize,

    #[arg(short = "t", long = "thread", default_value = "4")]
    pub num_threads: usize,

    #[arg(short = "o", long = "output")]
    pub output_path: Option<String>,
}

impl TracerArgs {
    pub fn from_std() -> Option<TracerArgs> {
        let raw_args: Vec<String> = env::args().collect();
        let arguments = TracerArgs::from_args(raw_args.iter().skip(1).map(String::as_str));
        match arguments {
            Err(error) => {
                use ParseError::*;
                match error {
                    HelpRequested(message) => eprintln!("{}", message),
                    UnknownFlag(message) => eprintln!("Unknown flag {}", message),
                    TooManyArgs => eprintln!("Too many args!"),
                    RequiredArgMissing(arg) => eprintln!("the argument {} is missing", arg),
                    MissingValue(arg) => eprintln!("{}'s value is missing", arg),
                    InvalidFlagValue(flag, value) => {
                        eprintln!("Invalid value {} for {}", value, flag)
                    }
                    InvalidArgValue(arg, value) => eprintln!("Invalid value {} for {}", value, arg),
                }
                None
            }
            Ok(args) => Some(args),
        }
    }
}
