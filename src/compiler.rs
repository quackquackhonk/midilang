use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::{LLVMBuilder, LLVMModule};
use log::debug;
use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr::null_mut;

use crate::parser::MidiAST;

pub enum MCompileError {
    LLVMError(String),
    UninitializedContext,
}

impl fmt::Debug for MCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LLVMError(msg) => write!(f, "LLVM raised an error {}", msg),
            Self::UninitializedContext => write!(f, "Compiler Context was not initialized!"),
        }
    }
}

pub type MCompileResult<T> = Result<T, MCompileError>;

// some wrappings on LLVM constructs
// Lots of code here is taken from wilfreds bfc compiler:
// https://github.com/Wilfred/bfc
const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

unsafe fn int1(val: u64) -> LLVMValueRef {
    LLVMConstInt(LLVMInt1Type(), val, LLVM_FALSE)
}

/// Convert this integer to LLVM's representation of a constant
/// integer.
unsafe fn int8(val: u64) -> LLVMValueRef {
    LLVMConstInt(LLVMInt8Type(), val, LLVM_FALSE)
}

/// Convert this integer to LLVM's representation of a constant
/// integer.
// TODO: this should be a machine word size rather than hard-coding 32-bits.
fn int32(val: u64) -> LLVMValueRef {
    unsafe { LLVMConstInt(LLVMInt32Type(), val, LLVM_FALSE) }
}

fn bool_type() -> LLVMTypeRef {
    unsafe { LLVMInt1Type() }
}

fn int8_type() -> LLVMTypeRef {
    unsafe { LLVMInt8Type() }
}

fn int32_type() -> LLVMTypeRef {
    unsafe { LLVMInt32Type() }
}

fn int64_type() -> LLVMTypeRef {
    unsafe { LLVMInt64Type() }
}

fn int8_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt8Type(), 0) }
}

fn void_type() -> LLVMTypeRef {
    unsafe { LLVMVoidType() }
}

/// A struct that keeps ownership of all the strings we've passed to
/// the LLVM API until we destroy the `LLVMModule`.
/// Wraps LLVM's builder class to provide a nicer API and ensure we
/// always dispose correctly.
pub struct MidiCompiler {
    builder: *mut LLVMBuilder,
    blocks: Option<(LLVMBasicBlockRef, LLVMBasicBlockRef)>,
    module: *mut LLVMModule,
    context: Option<MidiContext>,
    strings: Vec<CString>,
}

#[derive(Clone)]
struct MidiContext {
    cells_ptr: LLVMValueRef,
    cell_idx_ptr: LLVMValueRef,
    main_fn: LLVMValueRef,
}

impl MidiCompiler {
    fn new(module_name: &str, target_triple: Option<()>) -> Self {
        let module_name_cstr = CString::new(module_name).unwrap();
        let module_name_ptr = module_name_cstr.to_bytes_with_nul().as_ptr() as *const i8;
        // create module
        let llvm_module = unsafe { LLVMModuleCreateWithName(module_name_ptr) };
        // create builder
        let llvm_builder = unsafe { LLVMCreateBuilder() };
        let mut mm = MidiCompiler {
            builder: llvm_builder,
            blocks: None,
            module: llvm_module,
            context: None,
            strings: vec![module_name_cstr],
        };

        // TODO: add target triple stuff
        mm
    }

    fn init(&mut self, num_cells: u64) -> MCompileResult<()> {
        self.add_c_declarations();
        let main_fn = self.add_main_fn();
        let (init_bb, start_bb) = self.add_initial_blocks(main_fn);
        self.blocks = Some((init_bb, start_bb));
        self.position_builder_at_end(init_bb);
        let cells_ptr = self.allocate_cells(num_cells);
        let cell_idx_ptr = self.create_cell_idx_ptr();
        self.context = Some(MidiContext {
            cells_ptr,
            cell_idx_ptr,
            main_fn,
        });
        self.position_builder_at_end(start_bb);
        Ok(())
    }

    fn add_c_declarations(&mut self) {
        // add c delaractions to the module
        // &int8 malloc(int32)
        self.add_function("malloc", &mut [int32_type()], int8_ptr_type());
        // void free(&int8)
        self.add_function("free", &mut [int8_ptr_type()], void_type());
        // int32 putchar(int32)
        self.add_function("putchar", &mut [int32_type()], int32_type());
        // int32 getchar()
        self.add_function("getchar", &mut [], int32_type());
        // llvm.memset.p0i8.u64
        self.add_function(
            "llvm.memset.p0i8.i32",
            &mut [
                int8_ptr_type(),
                int8_type(),
                int32_type(),
                int32_type(),
                bool_type(),
            ],
            void_type(),
        );
    }

    /// Allocates necessary cells for the data tape
    /// returns the pointer to the data tape
    fn allocate_cells(&mut self, num_cells: u64) -> LLVMValueRef {
        unsafe {
            // char* cells = malloc(num_cells)
            let num_cells_llvm = int32(num_cells);
            let mut malloc_args = vec![num_cells_llvm];
            let cells_ptr = self.add_function_call("malloc", &mut malloc_args, "cells");

            // memset(cells, num_cells, 0, 0)
            let zero_i8 = int8(0);
            let one_i32 = int32(0);
            let false_i1 = int1(0);
            let mut memset_args = vec![cells_ptr, zero_i8, num_cells_llvm, one_i32, false_i1];
            self.add_function_call("llvm.memset.p0i8.i32", &mut memset_args, "");

            cells_ptr
        }
    }

    fn create_cell_idx_ptr(&mut self) -> LLVMValueRef {
        // char* cell_idx_ptr = cells;
        unsafe {
            // allocate stack space for the pointer
            let cell_idx_ptr = LLVMBuildAlloca(self.builder, int32_type(), self.new_string_ptr("cell_idx_ptr"));

            // set initial value to 0
            LLVMBuildStore(self.builder, int32(0), cell_idx_ptr);
            cell_idx_ptr
        }
    }

    fn add_main_fn(&mut self) -> LLVMValueRef {
        let mut main_args = vec![];
        unsafe {
            let main_ty = LLVMFunctionType(int32_type(), main_args.as_mut_ptr(), 0, LLVM_FALSE);
            LLVMAddFunction(self.module, self.new_string_ptr("main"), main_ty)
        }
    }

    fn add_initial_blocks(
        &mut self,
        main_fn: LLVMValueRef,
    ) -> (LLVMBasicBlockRef, LLVMBasicBlockRef) {
        unsafe {
            // This basic block is empty, but we will add a branch during
            // compilation according to InstrPosition.
            let init_bb = LLVMAppendBasicBlock(main_fn, self.new_string_ptr("init"));

            // We'll begin by appending instructions here.
            let start_bb = LLVMAppendBasicBlock(main_fn, self.new_string_ptr("start"));

            (init_bb, start_bb)
        }
    }

    fn cleanup(&mut self) -> MCompileResult<()> {
        // free cells datatype
        match &self.context {
            Some(xx) => {
                self.free_cells(xx.cells_ptr);
                unsafe {
                    self.build_return();
                }
            },
            None => return Err(MCompileError::UninitializedContext)
        }
        Ok(())
    }

    fn free_cells(&mut self, cells: LLVMValueRef) {
        // free the cells pointer
        unsafe {
            let mut free_args = vec![cells];
            self.add_function_call("free", &mut free_args, "free_cells");
        }
    }

    unsafe fn build_return(&mut self) {
        let zero = int32(0);
        LLVMBuildRet(self.builder, zero);
    }

    /// Wrapper around LLVMAddFunction for inserting function declarations into this module
    fn add_function(&mut self, fn_name: &str, fn_args: &mut [LLVMTypeRef], ret_ty: LLVMTypeRef) {
        unsafe {
            let fn_ty = LLVMFunctionType(
                ret_ty,
                fn_args.as_mut_ptr(),
                fn_args.len() as u32,
                LLVM_FALSE,
            );
            LLVMAddFunction(self.module, self.new_string_ptr(fn_name), fn_ty);
        }
    }

    unsafe fn add_function_call(
        &mut self,
        fn_name: &str,
        fn_args: &mut [LLVMValueRef],
        name: &str,
    ) -> LLVMValueRef {
        let fn_val = LLVMGetNamedFunction(self.module, self.new_string_ptr(fn_name));
        LLVMBuildCall(
            self.builder,
            fn_val,
            fn_args.as_mut_ptr(),
            fn_args.len() as u32,
            self.new_string_ptr(name),
        )
    }

    fn position_builder_at_end(&self, bb: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, bb);
        }
    }

    /// Create a new CString associated with this LLVMModule,
    /// and return a pointer that can be passed to LLVM APIs.
    /// Assumes s is pure-ASCII.
    fn new_string_ptr(&mut self, s: &str) -> *const i8 {
        self.new_mut_string_ptr(s)
    }

    // TODO: ideally our pointers wouldn't be mutable.
    fn new_mut_string_ptr(&mut self, s: &str) -> *mut i8 {
        let cstring = CString::new(s).unwrap();
        let ptr = cstring.as_ptr() as *mut _;
        self.strings.push(cstring);
        ptr
    }

    pub fn to_cstring(&self) -> CString {
        unsafe {
            // LLVM gives us a *char pointer, so wrap it in a CStr to mark it
            // as borrowed.
            let llvm_ir_ptr = LLVMPrintModuleToString(self.module);
            let llvm_ir = CStr::from_ptr(llvm_ir_ptr as *const _);

            // Make an owned copy of the string in our memory space.
            let module_string = CString::new(llvm_ir.to_bytes()).unwrap();

            // Cleanup borrowed string.
            LLVMDisposeMessage(llvm_ir_ptr);

            module_string
        }
    }
}

impl Drop for MidiCompiler {
    fn drop(&mut self) {
        // Rust requires that drop() is a safe function.
        unsafe {
            LLVMDisposeModule(self.module);
            LLVMDisposeBuilder(self.builder);
        }
    }
}

// taken from Wilfred's bfc project
struct TargetMachine {
    tm: LLVMTargetMachineRef
}

impl TargetMachine {
    fn new(target_triple: *const i8) -> MCompileResult<Self> {
        let mut target = null_mut();
        let mut err_msg_ptr = null_mut();
        unsafe {
            LLVMGetTargetFromTriple(target_triple, &mut target, &mut err_msg_ptr);
            if target.is_null() {
                // LLVM couldn't find a target triple with this name,
                // so it should have given us an error message.
                assert!(!err_msg_ptr.is_null());

                let err_msg_cstr = CStr::from_ptr(err_msg_ptr as *const _);
                let err_msg = std::str::from_utf8(err_msg_cstr.to_bytes()).unwrap();
                return Err(MCompileError::LLVMError(err_msg.to_owned()));
            }
        }

        // TODO: do these strings live long enough?
        // cpu is documented: http://llvm.org/docs/CommandGuide/llc.html#cmdoption-mcpu
        let cpu = CString::new("generic").unwrap();
        // features are documented: http://llvm.org/docs/CommandGuide/llc.html#cmdoption-mattr
        let features = CString::new("").unwrap();

        let target_machine;
        unsafe {
            target_machine = LLVMCreateTargetMachine(
                target,
                target_triple,
                cpu.as_ptr() as *const _,
                features.as_ptr() as *const _,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                LLVMRelocMode::LLVMRelocPIC,
                LLVMCodeModel::LLVMCodeModelDefault,
            );
        }

        Ok(TargetMachine { tm: target_machine })
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.tm);
        }
    }
}

pub fn init_llvm() {
    // TODO: I don't know how much of this I actually need
    unsafe {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();
    }
}

/// Compiles the given `MidiAST` into LLVM IR
pub fn compile_program(midi_program: MidiAST) -> MCompileResult<()> {
    debug!("Initializing LLVM...");
    init_llvm();
    debug!("Creating Module ...");
    let mut midimod = MidiCompiler::new("midilang", None);
    midimod.init(midi_program.highest_cell() as u64)?;
    midimod.cleanup()?;
    debug!("Parsing complete!");
    let ir_cstr = midimod.to_cstring();
    let ml_ir = String::from_utf8_lossy(ir_cstr.as_bytes());
    println!("{}", ml_ir);
    Ok(())
}

pub fn write_object_code(module: &mut MidiCompiler, path: &str) -> MCompileResult<()> {
    unsafe {
        let target_triple = LLVMGetTarget(module.module);
        let target_machine = TargetMachine::new(target_triple)?;

        let mut obj_error = module.new_mut_string_ptr("Writing object file failed.");
        let result = LLVMTargetMachineEmitToFile(
            target_machine.tm,
            module.module,
            module.new_string_ptr(path) as *mut i8,
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut obj_error,
        );

        if result != 0 {
            panic!("obj_error: {:?}", CStr::from_ptr(obj_error as *const _));
        }
    }
    Ok(())
} 
