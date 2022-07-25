
use std::collections::BinaryHeap;

use midly::MidiMessage;

/// Defines the Abstract Syntax Tree (AST) for midilang.
/// 
/// The Syntax corresponding to these instructions is as follows:
/// - `+` -> IncrementCell(...)
/// - `-` -> DecrementCell(...)
/// - `>` -> MovePointer { right: true, amount: ... }
/// - `<` -> MovePointer { right false, amount: ... }
/// - `.` -> Read
/// - `,` -> Write
/// - `[` -> JumpZero
/// - `]` -> JumpNotZero
/// 
/// A midilang Program is defined by a vector of MASTs.


use MASTNode::*;
pub enum MASTNode {
    IncrementCell(i32),
    DecrementCell(i32),
    MoveLeft(i32),
    MoveRight(i32),
    Read,
    Write, /// do we need to capture input?
    JumpZero,
    JumpNotZero
}

pub type MidiAST = Vec<MASTNode>;

pub type MParseResult = Result<MidiAST, MParseError>;

pub enum MParseError {
    NoTracks,
    IncompleteLoop,
    NonDiatonic,
}

// A key is a function turning a list of notes into an MASTNode
fn c_major(vals: Vec<u8>) -> Option<MASTNode> {
    // unwrap is safe, we will never deal with an empty vector
    let root = vals.get(0).unwrap() % 12;
    let mut amount: i32 = 1;
    let mut base = None;
    for vv in vals[1..].iter() {
        if let Some(bb) = base {
            amount = amount + (2_i32.pow(u32::from(vv - bb)));
        } else {
            base = Some(vv);
        }
    };
    match root {
        0 => Some(JumpNotZero),
        2 => Some(MoveLeft(amount)),
        4 => Some(MoveRight(amount)),
        5 => Some(DecrementCell(amount)),
        7 => Some(JumpZero),
        9 => Some(IncrementCell(amount)),
        11 => Some(if amount == 1 {Write} else {Read}),
        _ => None
    }
}

pub fn parse(midi: midly::Smf) -> MParseResult { 

    let mut prog = Vec::new();

    // TODO: Figure out what song the key is in, for now everything is in C major
    let program_key = c_major;

    if !midi.tracks.len() > 0 {
        return Err(MParseError::NoTracks)
    }

    let mut current_node = BinaryHeap::<u8>::new();

    for te in midi.tracks[0].iter() {

        if let midly::TrackEventKind::Midi{channel: _, message} = te.kind {
            if let MidiMessage::NoteOn{key, vel: _} = message {
                println!("{:?}", message);
                // if the note starts at the same time 
                if te.delta == 0 {
                    current_node.push(u8::from(key));
                }
                else {
                    match program_key(current_node.into_sorted_vec()) {
                        Some(node) => {
                            &prog.push(node);
                        },
                        None => return Err(MParseError::NonDiatonic)
                    }
                    current_node = BinaryHeap::<u8>::from([u8::from(key)]);
                }
            }
        }
    }

    Ok(prog)

}