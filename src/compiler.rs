use log::debug;

use crate::parser::{Cell, MidiAST, MidiInstruction, MidiInstructionKind::*};

/// Compiles the given `MidiAST` into LLVM IR
pub fn compile_program(midi_program: MidiAST) {
    debug!("Compiling ...");
    debug!("{midi_program:?}");
    println!("AAAAAAAAAAAAAA");
    // unimplemented!()
}
