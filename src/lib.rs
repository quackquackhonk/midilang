
use std::error::Error;
use std::io::Read;
use std::fs::{self, File};
use log::{info, error, debug};
use midly::num::u15;
use midly::num::u4;
use midly::num::u28;
use midly::num::u7;
use midly::{Smf, MidiMessage, TrackEvent, TrackEventKind, Format, Timing, Header, Track};

pub mod parse;
use crate::parse::{self, MParseError};

pub fn run(file_path: &str) -> Result<i32, Box<dyn Error>> {
    info!("Reading MIDI file from {}", &file_path);
    // read file
    let bytes = fs::read(file_path)?;
    let midi = Smf::parse(&bytes)?;

    // parse midi SMF into midi program AST
    let mprog = parse::parse(midi);
    info!("{:?}", mprog);
    if let Err(me) = mprog {
        match me {
            MParseError::NoTracks => error!("{} had no tracks to parse.", &file_path),
            MParseError::NonDiatonic => error!("Found non-diatonic note when parsing {}", &file_path),
            MParseError::UnclosedLoop(ls) => { 
                error!("Found unclosed loops when parsing {} at position(s) {:#?}", &file_path, ls);
            },
            MParseError::DanglingLoop(pos) => {
                error!("Found dangling close loop when parsing {} @ TrackEvent # {}", &file_path, pos)
            }
        }
        // return error code
        return Ok(1)
    }
    // llvm_ir::generate(mprog.unwrap());
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
            message: MidiMessage::NoteOn { key, vel: u7::from(127) } }
    }
}
fn make_off<'a>(key: u7) -> TrackEvent<'a> {
    TrackEvent {
        delta: u28::from(10),
        kind: TrackEventKind::Midi { 
            channel: u4::from(1),
            message: MidiMessage::NoteOff { key, vel: u7::from(127) } }
    }
}

// Converts a brainfuck program into a MIDIlang program in Smf
pub fn from_brainfuck(bf_file_path: &str) -> Result<(), Box<dyn Error>> {
    info!("Converting BF file {} to Standard Midi Format...", &bf_file_path);
    let mut ml_file_path = bf_file_path.strip_suffix('.')
                                   .unwrap_or(bf_file_path)
                                   .to_owned();
    ml_file_path.push_str(".mid");
    let mut bf_file = File::open(bf_file_path)?;
    let mut bf_program = String::new();
    bf_file.read_to_string(&mut bf_program)?;

    let ml_file = File::options().append(false)
                                 .write(true)
                                 .create(true)
                                 .open(&ml_file_path)?;
    let mut ml_prog = Smf::new(Header::new(Format::Parallel, Timing::Metrical(u15::from(480))));
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
                continue
            },
            _ => continue
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


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
