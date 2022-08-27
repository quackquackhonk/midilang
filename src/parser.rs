
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::num::Wrapping;

use log::{debug, info};
use midly::MidiMessage;

/// Defines the Abstract Syntax Tree (AST) for midilang.
/// 
/// The Syntax corresponding to these instructions is as follows:
/// - `+` -> IncrementCell(...)
/// - `-` -> IncrementCell(...) (constructed with a negated argument)
/// - `>` -> MovePointer { right: true, amount: ... }
/// - `<` -> MovePointer { right false, amount: ... }
/// - `.` -> OutputCell
/// - `,` -> InputCell
/// - `[` -> Loop {}
/// - `]` -> JumpNotZero
/// 
/// A midilang Program is defined by a vector of MASTs.

use MidiInstructionKind::*;

/// BF cells are exactly one byte
pub type Cell = Wrapping<i8>;

/// Range for keeping track of positions in code
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Position {
    start: usize,
    end: usize
}

impl Position {
    fn new(start: usize, end: usize) -> Self {
        Position{ start, end }
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(f, "({})", self.start)
        } else {
            write!(f, "({},{})", self.start, self.end)
        }
    }
}


/// Our instruction datatype
        // Loops with position: `None` are used to represent closed loops
        // Loops with position: `Some(_)` are used for open loops
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MidiInstruction {
    position: Option<Position>,
    instruction: MidiInstructionKind
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MidiInstructionKind {
    IncrementCell {
        amount: Cell,
    },
    MovePointer {
        amount: isize,
    },
    OutputCell,
    InputCell,
    Loop {
        body: MidiAST
    }
}

impl MidiInstruction {

    fn new_inc(position: Option<Position>, amount: Cell) -> Self {
        MidiInstruction {
            position,
            instruction: IncrementCell { amount }
        }
    }

    fn new_move(position: Option<Position>, amount: isize) -> Self {
        MidiInstruction {
            position,
            instruction: MovePointer { amount }
        }
    }

    fn new_close_loop() -> Self {
        MidiInstruction {
            position: None,
            instruction: Loop { body: vec![] }
        }
    }

    fn new_open_loop() -> Self {
        MidiInstruction {
            position: Some(Position::new(0, 0)),
            instruction: Loop { body: vec![] }
        }
    }

    fn new_output(position: Option<Position>) -> Self {
        MidiInstruction {
            position,
            instruction: OutputCell
        }
    }

    fn new_input(position: Option<Position>) -> Self {
        MidiInstruction {
            position,
            instruction: InputCell
        }
    }

    fn set_position(&mut self, new_pos: Position) {
        self.position = Some(new_pos);
    }
}


// pub type MidiAST = Vec<MidiInstruction>;
pub struct MidiASTBuilder {
    body: MidiAST,
    size: usize,
    loop_stack: Vec<(MidiAST, usize)>
}

impl MidiASTBuilder {
    pub fn new() -> Self {
        MidiASTBuilder {
            body: Vec::<MidiInstruction>::new(),
            size: 0,
            loop_stack: vec![],
        }        
    }

    pub fn push(&mut self, mut inst: MidiInstruction) -> MParseResult<()> {
        match inst {
            MidiInstruction { position: Some(_), instruction: Loop {..}} => {
                // open loop 
                self.loop_stack.push((self.body.drain(..).collect(), self.size));
                self.body = vec![];
            },
            MidiInstruction { position: None, instruction: Loop {..}} => {
                // close loop
                if let Some((mut before_loop, loop_start)) = self.loop_stack.pop() {
                    before_loop.push(MidiInstruction {
                        position: Some(Position::new(loop_start, self.size)),
                        instruction: Loop {
                            body: self.body.to_owned()
                        }
                    });
                    self.body = before_loop;
                }
                else {
                    return Err(MParseError::DanglingLoop(Position::new(self.size, self.size)));
                }
            },
            _ => {
                inst.set_position(Position::new(self.size, self.size));
                self.body.push(inst);
            }
        }
        self.size += 1;
        Ok(())
    }

    pub fn into_mast(&self) -> MParseResult<MidiAST> {
        if self.loop_stack.is_empty() {
            Ok(self.body.to_owned())
        } else {
            let loops = self.loop_stack.iter()
                                       .map(|(_b, start)| Position::new(*start, *start))
                                       .collect();
            Err(MParseError::UnclosedLoop(loops))
        }
    }
}

impl Default for MidiASTBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub type MidiAST = Vec<MidiInstruction>;

pub type MParseResult<T> = Result<T, MParseError>;

#[derive(PartialEq)]
pub enum MParseError {
    NoTracks,
    UnclosedLoop(Vec<Position>),
    DanglingLoop(Position),
    NonDiatonic,
}

impl Debug for MParseError {
    // TODO: Fix error descriptions
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTracks => write!(f, "File has no tracks to parse!"),
            Self::UnclosedLoop(poss) => write!(f, "Unclosed loops starting at: {:?}", poss),
            Self::DanglingLoop(pos) => write!(f, "Dangling loops starting at: {:?}", pos),
            Self::NonDiatonic => write!(f, "Non Diatonic note found")
        }
    }
}

fn parse_chord<F: Fn(u8, i8) -> MParseResult<MidiInstruction>>(vals: Vec<u8>, key: &F) -> MParseResult<MidiInstruction> {
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
                if tmp > 8 {
                    break;
                }
                let to_add = 2_i8.pow(u32::from(vv - bb - 1));
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


fn c_major(root: u8, arg: i8) -> MParseResult<MidiInstruction> {
    let position = None;
    match root {
        0 => Ok(MidiInstruction::new_close_loop()),
        2 => Ok(MidiInstruction::new_move(position, -isize::from(arg))),
        4 => Ok(MidiInstruction::new_move(position, isize::from(arg))),
        5 => Ok(MidiInstruction::new_inc(position, Wrapping(-arg))),
        7 => Ok(MidiInstruction::new_open_loop()),
        9 => Ok(MidiInstruction::new_inc(position, Wrapping(arg))),
        11 if arg == 1 => Ok(MidiInstruction::new_input(position)),
        11 => Ok(MidiInstruction::new_output(position)),
        _ => Err(MParseError::NonDiatonic)
    }
}

pub fn parse(midi: midly::Smf) -> MParseResult<MidiAST> { 

    info!("Starting to parse MIDI file...");

    let mut ast_builder = MidiASTBuilder::new();

    // TODO: Figure out what song the key is in, for now everything is in C major
    let program_key = |xx| parse_chord(xx, &c_major);

    if midi.tracks.is_empty() {
        return Err(MParseError::NoTracks)
    }

    let mut current_node = BinaryHeap::<u8>::new();
    debug!("MIDI File Header: {:?}", midi.header);
    for track in midi.tracks {
        let mut notes_on: i32 = 0;
        for (_, te) in track.iter().enumerate() {
            if let midly::TrackEventKind::Midi{channel: _, message} = te.kind {
                debug!("Processing {:?}", message);
                match message {
                    MidiMessage::NoteOn{key, vel: _} => {
                        debug!("{} pressed: {} -> {}", key, notes_on, notes_on + 1);
                        current_node.push(u8::from(key));
                        notes_on += 1;
                    },
                    MidiMessage::NoteOff{key, ..} => {
                        debug!("{} released: {} -> {}", key, notes_on, notes_on -1);
                        notes_on -= 1;

                        if notes_on == 0 {
                            debug!("All notes are off, parsing instruction...");
                            debug!("parsing {:?}", current_node);
                            match program_key(current_node.into_sorted_vec()) {
                                Ok(node) => {
                                    debug!("Parsing successful: {:?}", node);
                                    ast_builder.push(node)?;

                                },
                                Err(err) => return Err(err) 
                            }
                            current_node = BinaryHeap::<u8>::new();
                        }
                    },
                    _ => {
                        debug!("Ignoring non-midi message...");
                    }
                }
            }
        }
    }

    ast_builder.into_mast()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_chord_c_major_no_args() {
        let position = None;
        let key = |xx| parse_chord(xx, &c_major);
        let tonic = Vec::from([0]);
        let supertonic = Vec::from([2]);
        let mediant = Vec::from([4]);
        let subdominant = Vec::from([5]);
        let dominant = Vec::from([7]);
        let submediant = Vec::from([9]);
        let leading_tone = Vec::from([11]);
        let non_diatonic = Vec::from([8]);
        assert_eq!(key(tonic).unwrap(), MidiInstruction::new_close_loop());
        assert_eq!(key(supertonic).unwrap(), MidiInstruction::new_move(position, -1));
        assert_eq!(key(mediant).unwrap(), MidiInstruction::new_move(position, 1));
        assert_eq!(key(subdominant).unwrap(), MidiInstruction::new_inc(position, Wrapping(-1)));
        assert_eq!(key(dominant).unwrap(), MidiInstruction::new_open_loop());
        assert_eq!(key(submediant).unwrap(), MidiInstruction::new_inc(position, Wrapping(1)));
        assert_eq!(key(leading_tone).unwrap(), MidiInstruction::new_input(position));
        assert_eq!(key(non_diatonic).unwrap_err(), MParseError::NonDiatonic);
    }

    #[test]
    fn c_major_args() {
        let position = None;
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
        assert_eq!(key(tonic_chord).unwrap(), MidiInstruction::new_close_loop());
        assert_eq!(key(supertonic_chord).unwrap(), MidiInstruction::new_move(position, -16));
        assert_eq!(key(mediant_chord).unwrap(), MidiInstruction::new_move(position, 10));
        assert_eq!(key(subdominant_chord).unwrap(), MidiInstruction::new_inc(position, Wrapping(-2)));
        assert_eq!(key(dominant_chord).unwrap(), MidiInstruction::new_open_loop());
        assert_eq!(key(submediant_chord).unwrap(), MidiInstruction::new_inc(position, Wrapping(4)));
        assert_eq!(key(leading_tone_octave).unwrap(), MidiInstruction::new_input(position));
        assert_eq!(key(leading_tone_chord).unwrap(), MidiInstruction::new_output(position));
        assert_eq!(key(non_diatonic).unwrap_err(), MParseError::NonDiatonic);
    }

}
