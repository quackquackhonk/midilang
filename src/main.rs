use clap::Parser;
use env_logger::{self, Builder, Target, WriteStyle};
use log::{self, error, info, LevelFilter};

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
    bf: Option<String>,

    #[clap(short, long, action)]
    debug: bool,

    #[clap(short, long, action)]
    verbose: bool,

    #[clap(long, action)]
    dump_llvm: bool,
}

fn main() {
    let cli_args = MidilangCli::parse();

    Builder::new()
        .filter(
            None,
            if cli_args.debug {
                LevelFilter::Trace
            } else {
                LevelFilter::Warn
            },
        )
        .write_style(WriteStyle::Auto)
        .target(if cli_args.verbose {
            Target::Stdout
        } else {
            Target::Stderr
        })
        .init();

    if let Some(bf) = cli_args.bf {
        match midilang::from_brainf(&bf) {
            Err(e) => error!("Error when parsing BF file: {}", e),
            Ok(_) => info!("BF File parsed successfully!"),
        }
    }
    if let Some(path) = cli_args.file_name {
        match midilang::compile_file(&path) {
            Err(e) => error!("Application Error {}", e),
            Ok(_) => info!("Ran successfully!"),
        }
    }
}
