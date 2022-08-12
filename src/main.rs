use clap::Parser;
use log::{self, LevelFilter, error};
use env_logger::{self, Builder, WriteStyle, Target};
use midilang::parse::MParseError;

/// A Program to compile midi into executable code
#[derive(Parser, Debug)]
#[clap(name = "Midi Lang")]
#[clap(author = "sahanatankala@gmail.com")]
#[clap(author = "0.1")]
#[clap(about = "An assembly compiler for MIDI files", long_about = None)]
struct MidilangCli {
    
    #[clap(short = 'm', long = "midi", value_parser, value_name = "FILE")]
    file_name: Option<String>,

    #[clap(long, value_parser, value_name = "BF_FILE")]
    brainfuck: Option<String>,

    #[clap(short, long, action)]
    debug: bool,

    #[clap(short, long, action)]
    verbose: bool
}

fn main() {
    let cli_args = MidilangCli::parse();

    let log_builder = Builder::new()
            .filter(None, if cli_args.debug {LevelFilter::Trace} else {LevelFilter::Warn})
            .write_style(WriteStyle::Auto)
            .target(if cli_args.verbose {Target::Stdout} else {Target::Stderr} )
            .init();

    if let Some(bf) = cli_args.brainfuck {
        midilang::from_brainfuck(&bf);
    }
    if let Some(path) = cli_args.file_name {
        match midilang::run(&path) {
            Err(e) => error!("Application Error {}", e),
            Ok(x) => println!("Ran successfully!")
        }
    }
}
