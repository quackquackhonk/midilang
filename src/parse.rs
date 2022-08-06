
use std::collections::BinaryHeap;

use midly::MidiMessage;

/// Defines the Abstract Syntax Tree (AST) for midilang.
/// 
/// The Syntax corresponding to these instructions is as follows:
/// - `+` -> IncrementCell(...)
/// - `-` -> DecrementCell(...)
/// - `>` -> MovePointer { right: true, amount: ... }
/// - `<` -> MovePointer { right false, amount: ... }
/// - `.` -> OutputCell
/// - `,` -> InputCell
/// - `[` -> JumpZero
/// - `]` -> JumpNotZero
/// 
/// A midilang Program is defined by a vector of MASTs.

use MASTNode::*;
#[derive(Debug, PartialEq)]
pub enum MASTNode {
    IncrementCell(u32),
    DecrementCell(u32),
    MoveLeft(u32),
    MoveRight(u32),
    OutputCell,
    InputCell, /// do we need to capture input?
    JumpZero,
    JumpNotZero
}

pub type MidiAST = Vec<MASTNode>;

pub type MParseResult<T> = Result<T, MParseError>;

#[derive(Debug, PartialEq)]
pub enum MParseError {
    NoTracks,
    UnclosedLoop(usize),
    DanglingLoop(usize),
    NonDiatonic,
}

fn parse_chord<F: Fn(u8, u32) -> MParseResult<MASTNode>>(vals: Vec<u8>, key: &F) -> MParseResult<MASTNode> {
    // unwrap is safe, we will never deal with an empty vector
    let root = vals.get(0).unwrap() % 12;
    let mut arg = None;
    let mut base = None;
    let mut prev = root;
    for vv in vals[1..].iter() {
        if prev != *vv {
            if let Some(bb) = base {
                let tmp = vv - bb - 1;
                // Need to protect against overflow
                if tmp > 32 {
                    break;
                }
                let to_add = 2_u32.pow(u32::from(vv - bb - 1));
                arg = arg.map_or(Some(to_add), |xx| Some(xx + to_add));
            } else {
                base = Some(vv);
                prev = *vv
            }
        }
    };
    let amount = arg.unwrap_or(1);
    key(root, amount)
}

fn c_major(root: u8, arg: u32) -> MParseResult<MASTNode> {
    match root {
        0 => Ok(JumpNotZero),
        2 => Ok(MoveLeft(arg)),
        4 => Ok(MoveRight(arg)),
        5 => Ok(DecrementCell(arg)),
        7 => Ok(JumpZero),
        9 => Ok(IncrementCell(arg)),
        11 if arg == 1 => Ok(InputCell),
        11 => Ok(OutputCell),
        _ => Err(MParseError::NonDiatonic)
    }
}

pub fn parse(midi: midly::Smf) -> MParseResult<MidiAST> { 

    let mut prog = Vec::new();

    // TODO: Figure out what song the key is in, for now everything is in C major
    let program_key = |xx| parse_chord(xx, &c_major);

    if midi.tracks.is_empty() {
        return Err(MParseError::NoTracks)
    }

    let mut current_node = BinaryHeap::<u8>::new();
    let mut loop_stack = Vec::<usize>::new();
    for track in midi.tracks {
        let mut notes_on: i32 = 0;
        for (idx, te) in track.iter().enumerate() {
            if let midly::TrackEventKind::Midi{channel: _, message} = te.kind {
                println!("Found {:?}", message);
                match message {
                    MidiMessage::NoteOn{key, vel: _} => {
                        println!("\t\tpushing {:?} into bh", key);
                        current_node.push(u8::from(key));
                        notes_on += 1;
                        println!("\t\tnum on: {}", notes_on);
                    },
                    MidiMessage::NoteOff{..} => {
                        notes_on -= 1;
                        println!("\t\tnum on: {}", notes_on);
                        if notes_on == 0 {
                            println!("\t\ttrying to parse {:?}", current_node);
                            match program_key(current_node.into_sorted_vec()) {
                                Ok(node) => {
                                    match node {
                                        JumpNotZero => {
                                            if loop_stack.pop().is_none() {
                                                return Err(MParseError::DanglingLoop(idx))
                                            }
                                            prog.push(node)
                                        },
                                        JumpZero => {
                                            loop_stack.push(idx);
                                            prog.push(node)
                                        }
                                        _ => prog.push(node)
                                    }
                                },
                                Err(err) => return Err(err) 
                            }
                            current_node = BinaryHeap::<u8>::new();
                        }
                    },
                    // _ => ()
                    _ => {
                        println!("\tIgnoring...")
                    }
                }
            }
        }
    }
    if !loop_stack.is_empty() {
        return Err(MParseError::UnclosedLoop(loop_stack.pop().unwrap()))
    }
    Ok(prog)

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_chord_c_major_no_args() {
        let key = |xx| parse_chord(xx, &c_major);
        let tonic = Vec::from([0]);
        let supertonic = Vec::from([2]);
        let mediant = Vec::from([4]);
        let subdominant = Vec::from([5]);
        let dominant = Vec::from([7]);
        let submediant = Vec::from([9]);
        let leading_tone = Vec::from([11]);
        let non_diatonic = Vec::from([8]);
        assert_eq!(key(tonic).unwrap(), JumpNotZero);
        assert_eq!(key(supertonic).unwrap(), MoveLeft(1));
        assert_eq!(key(mediant).unwrap(), MoveRight(1));
        assert_eq!(key(subdominant).unwrap(), DecrementCell(1));
        assert_eq!(key(dominant).unwrap(), JumpZero);
        assert_eq!(key(submediant).unwrap(), IncrementCell(1));
        assert_eq!(key(leading_tone).unwrap(), InputCell);
        assert_eq!(key(non_diatonic).unwrap_err(), MParseError::NonDiatonic);
    }

    #[test]
    fn c_major_args() {
        let key = |xx| parse_chord(xx, &c_major);
        // ignores arguments
        let tonic_chord = Vec::from([0, 12, 16, 18]);
        // 10000b = 16
        let supertonic_chord = Vec::from([26, 33, 38]);
        // 1010b = 10
        let mediant_chord = Vec::from([40, 44, 46, 48]);
        // 10b = 2
        let subdominant_chord = Vec::from([17, 29, 31]);
        // ignores arguments
        let dominant_chord = Vec::from([7, 100]);
        // 100b = 4
        let submediant_chord = Vec::from([9, 27, 30]);
        // leading tone with 1 other note = Write
        // leading tone with >=2 other notes = Read
        let leading_tone_octave = Vec::from([11, 23]);
        let leading_tone_chord = Vec::from([11, 23, 29]);
        // ignores arguments
        let non_diatonic = Vec::from([8, 10, 22]);
        assert_eq!(key(tonic_chord).unwrap(), JumpNotZero);
        assert_eq!(key(supertonic_chord).unwrap(), MoveLeft(16));
        assert_eq!(key(mediant_chord).unwrap(), MoveRight(10));
        assert_eq!(key(subdominant_chord).unwrap(), DecrementCell(2));
        assert_eq!(key(dominant_chord).unwrap(), JumpZero);
        assert_eq!(key(submediant_chord).unwrap(), IncrementCell(4));
        assert_eq!(key(leading_tone_octave).unwrap(), InputCell);
        assert_eq!(key(leading_tone_chord).unwrap(), OutputCell);
        assert_eq!(key(non_diatonic).unwrap_err(), MParseError::NonDiatonic);
    }

}
