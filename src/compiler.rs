
use inkwell::execution_engine::ExecutionEngine;
use log::debug;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

use crate::parser::{MidiAST, MidiInstruction, MidiInstructionKind::*};

struct MidiCompiler<'a, 'ctx> {
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    module: &'a Module<'ctx>,
    execution_engine: ExecutionEngine<'ctx>
}

impl<'a, 'ctx> MidiCompiler<'a, 'ctx> {

    fn compile_program(&mut self, midi_program: &MidiAST) {
        for inst in midi_program.iter() {
            // self.compile_instruction(&inst)
            unimplemented!()
        }
    }

    fn compile_instruction(&mut self, midi_inst: &MidiInstruction) {
        match &midi_inst.instruction {
            IncrementCell { amount } => todo!(),
            MovePointer { amount } => todo!(),
            Loop { body } => todo!(),
            OutputCell => todo!(),
            InputCell => todo!()
        }
    }
}

/// Compiles the given `MidiAST` into LLVM IR
pub fn compile_program(midi_program: MidiAST) {
    debug!("Compiling ...");
    debug!("{midi_program:?}");
    // unimplemented!()
}
