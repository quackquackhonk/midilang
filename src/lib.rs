use log::{debug, error, info};
use midly::num::{u15, u28, u4, u7};
use midly::{Format, Header, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;

pub mod compiler;
pub mod parser;
mod utils;
// use crate::parser::MParseError;

// compiles
pub fn compile_file(file_path: &str) -> Result<i32, Box<dyn Error>> {
    info!("Reading MIDI file from {}", &file_path);
    // read file
    let bytes = fs::read(file_path)?;
    let midi = Smf::parse(&bytes)?;

    // parse midi SMF into midi program AST
    if let Err(mperr) = parser::parse(midi) {
        error!("Error when parsing file: {:?}", mperr);
        return Ok(1);
    }

    // compiler::compile(midi_program);
    Ok(0)
}

// fn run_interactive() -> Result<i32, Box<dyn Error>> {
//     unimplemented!()
// }

fn make_on<'a>(key: u7) -> TrackEvent<'a> {
    TrackEvent {
        delta: u28::from(10),
        kind: TrackEventKind::Midi {
            channel: u4::from(1),
            message: MidiMessage::NoteOn {
                key,
                vel: u7::from(127),
            },
        },
    }
}
fn make_off<'a>(key: u7) -> TrackEvent<'a> {
    TrackEvent {
        delta: u28::from(10),
        kind: TrackEventKind::Midi {
            channel: u4::from(1),
            message: MidiMessage::NoteOff {
                key,
                vel: u7::from(127),
            },
        },
    }
}

// Converts a brainf program into a MIDIlang program in Smf
pub fn from_brainf(bf_file_path: &str) -> Result<(), Box<dyn Error>> {
    info!(
        "Converting BF file {} to Standard Midi Format...",
        &bf_file_path
    );
    let ml_file_path = utils::midi_name(bf_file_path);
    let mut bf_file = File::open(bf_file_path)?;
    let mut bf_program = String::new();
    bf_file.read_to_string(&mut bf_program)?;

    let ml_file = File::options()
        .append(false)
        .write(true)
        .create(true)
        .open(&ml_file_path)?;
    let mut ml_prog = Smf::new(Header::new(
        Format::Parallel,
        Timing::Metrical(u15::from(480)),
    ));

    // TODO: Add meta track information
    ml_prog.tracks.push(Track::new()); // meta track is idx 0

    ml_prog.tracks.push(Track::new()); // program track is [1]
    for inst in bf_program.chars() {
        let key = match inst {
            ']' => 0,
            '<' => 2,
            '>' => 4,
            '-' => 5,
            '[' => 7,
            '+' => 9,
            ',' => 11,
            '.' => {
                // need to add simultaneous notes to make parses recognize output char
                ml_prog.tracks[1].push(make_on(u7::from(11)));
                ml_prog.tracks[1].push(make_on(u7::from(15)));
                ml_prog.tracks[1].push(make_on(u7::from(18)));
                ml_prog.tracks[1].push(make_off(u7::from(18)));
                ml_prog.tracks[1].push(make_off(u7::from(15)));
                ml_prog.tracks[1].push(make_off(u7::from(11)));
                continue;
            }
            _ => continue,
        };
        ml_prog.tracks[1].push(make_on(u7::from(key)));
        ml_prog.tracks[1].push(make_off(u7::from(key)));
    }

    debug!("BF program parsed into:");
    debug!("{:#?}", ml_prog);
    if let Err(e) = ml_prog.write_std::<_>(ml_file) {
        error!("Error when writing SMF to {}: {}", &ml_file_path, e);
    }
    info!("BF parsing successful!");
    Ok(())
}
