
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;
use log::debug;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

use crate::parser::{MidiAST, MidiInstruction, MidiInstructionKind::*, Cell};

struct MidiCompiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>
}

impl<'ctx> MidiCompiler<'ctx> {

    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("midilang");
        let builder = context.create_builder();
        MidiCompiler { context, module, builder }
    }

    fn init(&self) {
        self.add_c_declarations();
        self.add_main_fn();
    }

    fn add_c_declarations(&self) {
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::Generic);
        let void_type = self.context.void_type();
        // need:
        // - addition function
        // - malloc function
        let malloc_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("malloc", malloc_type, None);
        // - free function
        let free_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("free", free_type, None);
        // - getchar function
        let getchar_ty = i32_type.fn_type(&[i32_type.into()], false);
        self.module.add_function("getchar", getchar_ty, None);
        // - putchar function
        let putchar_ty = i32_type.fn_type(&[], false);
        self.module.add_function("putchar", putchar_ty, None);
    }
    
    fn add_main_fn(&self) {
        // add main function
        let i32_ty = self.context.i32_type();
        let main_fn_ty = i32_ty.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_ty, None);

        // create basic block to start the main function
        let main_bb = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_bb);
    }

    fn compile_program(&mut self, midi_program: &MidiAST) {
        for inst in midi_program.iter() {
            // Setup Context, Builder, Module
            self.compile_instruction(inst);
            unimplemented!()
        }
    }

    fn compile_instruction(&mut self, midi_inst: &MidiInstruction) {
        match &midi_inst.instruction {
            IncrementCell { amount } => self.compile_increment(amount),
            MovePointer { amount } => self.compile_move(amount),
            Loop { body } => self.compile_loop(body),
            OutputCell => self.compile_output(),
            InputCell => self.compile_input()
        }
    }

    fn compile_increment(&self, amount: &Cell) {
        todo!();
    }

    fn compile_move(&self, amount: &isize) {
        todo!();
    }

    fn compile_input(&self) {
        todo!();
    }

    fn compile_output(&self) {
        todo!();
    }

    fn compile_loop(&self, body: &MidiAST) {
        todo!();
    }


}

/// Compiles the given `MidiAST` into LLVM IR
pub fn compile_program(midi_program: MidiAST) {

    debug!("Compiling ...");
    debug!("{midi_program:?}");

    let context = Context::create();
    let midi_comp = MidiCompiler::new(&context);
    midi_comp.init();


    unimplemented!()
}
