
use std::{io, fs, error::Error};
use midly::Smf;
use midly::TrackEventKind::{Midi, Meta};
use midly::MidiMessage::NoteOn;

mod parse;

pub fn run(file_path: &str) -> Result<i32, Box<dyn Error>> {
    println!("Reading MIDI from {}", &file_path);
    let bytes = fs::read(file_path)?;
    let midi = Smf::parse(&bytes)?;
    let mprog = parse::parse(midi);

    Ok(1)
}

pub fn run_interactive() -> Result<i32, Box<dyn Error>> {
    unimplemented!()
}


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
