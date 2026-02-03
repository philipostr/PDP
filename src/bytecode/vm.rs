use core::panic;
use std::error::Error;
use std::fmt::Display;

use colored::Colorize;

use super::OpCode;
use super::objects::ObjectRef;
use crate::bytecode::objects::{
    Class, CodeObject, CompiledFunction, FrozenGenerator, FunctionType, Object,
};
use crate::bytecode::{BytecodeEmitter, std_lib};
use crate::objref;
use crate::util::Map;

#[inline(always)]
fn insufficient_items(instr: &str) -> String {
    format!("{instr} used with insufficient items on the stack")
}

#[derive(Debug)]
pub struct RuntimeError {
    pub msg: String,
}

impl RuntimeError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for RuntimeError {}

#[derive(Debug, Default)]
pub struct VM {
    constants_pool: Vec<ObjectRef>,
    globals: Map<ObjectRef>,
    builtins: Map<ObjectRef>,
    classes: Vec<Class>,
    frame_stack: Vec<Frame>,
    eval_stack: Vec<ObjectRef>,
    temp_stack: Vec<ObjectRef>,
    called_python_func: bool,
}

impl VM {
    pub fn new(module: BytecodeEmitter) -> Self {
        let mut vm = Self::default();

        let (instructions, _, Some(constants_pool)) = module.dissolve() else {
            panic!("Called VM::new() with non-root emitter");
        };
        vm.constants_pool = constants_pool;
        vm.frame_stack.push(Frame::new(instructions, 0));
        vm
    }

    pub fn pop_tos(&mut self) -> ObjectRef {
        self.eval_stack.pop().unwrap()
    }

    pub fn push_tos(&mut self, tos: ObjectRef) {
        self.eval_stack.push(tos);
    }

    pub fn swap_tos(&mut self) {
        let len = self.eval_stack.len();
        if len < 2 {
            panic!("{}", insufficient_items("SWAP_TOP"));
        }
        self.eval_stack.swap(len - 1, len - 2);
    }

    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    pub fn start(&mut self /*debug: Debug*/) {
        // Register builtin functions
        self.builtins.insert("iter".to_string(), std_lib::iter_());
        self.builtins.insert("next".to_string(), std_lib::next_());
        self.builtins.insert("print".to_string(), std_lib::print_());
        self.builtins.insert("bool".to_string(), std_lib::bool_());
        self.builtins.insert("len".to_string(), std_lib::len_());

        // Initialize and register builtin classes
        // Order based on Object::class_idx()
        self.classes.push(std_lib::none::init_class());
        self.classes.push(std_lib::number::init_class());
        self.classes.push(std_lib::boolean::init_class());
        self.classes.push(std_lib::string::init_class());
        self.classes.push(std_lib::list::init_class());
        self.classes.push(std_lib::set::init_class());
        self.classes.push(std_lib::dict::init_class());
        self.classes.push(std_lib::code::init_class());
        self.classes.push(std_lib::function::init_class());
        self.classes.push(std_lib::generator::init_class());

        // Finally run the code!
        while let Some(frame) = self.frame_stack.last() {
            if let Err(e) = self.execute_opcode(frame.next_instruction()) {
                eprintln!("{} {e}", "error:".red().bold());
                return;
            }
        }
    }

    fn execute_opcode(&mut self, instruction: OpCode) -> Result<(), RuntimeError> {
        let mut inc_ip = true;

        // dbg!(&instruction);
        // dbg!(&self.eval_stack);

        match instruction {
            OpCode::NOP => {}
            OpCode::POP_TOP => {
                self.eval_stack.pop();
            }
            OpCode::SWAP_TOP => {
                self.swap_tos();
            }
            OpCode::DUP_TOP => {
                self.eval_stack.push(
                    self.eval_stack
                        .last()
                        .expect(&insufficient_items("DUP_TOP"))
                        .clone(),
                );
            }
            OpCode::INV_TOP => {
                let tos = self
                    .eval_stack
                    .last()
                    .expect(&insufficient_items("INV_TOP"))
                    .clone();
                let inv_method = tos.borrow().class(&self.classes).attr("__inv__")?;

                self.eval_stack.push(inv_method);
                self.handle_callable_object("__inv__", 1)?;
            }
            OpCode::JUMP_FORWARD(n) => {
                inc_ip = false;
                self.top_frame().inc_ip(n);
            }
            OpCode::JUMP_IF_FALSE(n) => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("JUMP_IF_FALSE"));
                if let Object::Boolean(b) = *tos.borrow() {
                    if !b {
                        inc_ip = false;
                        self.top_frame().inc_ip(n);
                    }
                } else {
                    panic!("TOS must be a boolean when using JUMP_IF_FALSE")
                }
            }
            OpCode::JUMP_IF_TRUE(n) => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("JUMP_IF_TRUE"));
                if let Object::Boolean(b) = *tos.borrow() {
                    if b {
                        inc_ip = false;
                        self.top_frame().inc_ip(n);
                    }
                } else {
                    panic!("TOS must be a boolean when using JUMP_IF_TRUE")
                }
            }
            OpCode::JUMP_ABSOLUTE(n) => {
                inc_ip = false;
                self.top_frame().set_ip(n);
            }
            OpCode::MAKE_GENERATOR => {
                std_lib::iter(self)?;
            }
            OpCode::FOR_ITER(n) => {
                inc_ip = false;
                let tos = self
                    .eval_stack
                    .last()
                    .expect(&insufficient_items("FOR_ITER"))
                    .clone();
                let Object::Generator(ref mut generator) = *tos.borrow_mut() else {
                    return Err(RuntimeError::new(
                        "PDP does not support custom iterator classes in for loops yet",
                    ));
                };

                if generator.is_done() {
                    self.top_frame().inc_ip(n);
                } else {
                    self.top_frame().inc_ip(1); // Must be done before pushing a new frame
                    self.frame_stack
                        .push(generator.as_frame().with_offset(self.eval_stack.len()));
                    self.eval_stack.extend_from_slice(generator.eval_stack());
                }
            }
            OpCode::STORE_LOCAL(n) => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("STORE_LOCAL"));
                self.top_frame().set_local(n, tos);
            }
            // TODO: GH-10
            OpCode::STORE_DEREF(_) => todo!(),
            OpCode::STORE_GLOBAL(n) => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("STORE_GLOBAL"));

                let name = self.constants_pool[n].clone();
                let Object::String(ref name) = *name.borrow() else {
                    panic!("Constant object {n} expected to be a string, but is not");
                };

                self.globals.insert(name.clone(), tos);
            }
            // TODO: GH-9
            OpCode::STORE_ATTR(_) => todo!(),
            OpCode::STORE_ACCESS => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("STORE_ACCESS"));
                let tos1 = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("STORE_ACCESS"));
                let tos2 = self
                    .eval_stack
                    .last()
                    .expect(&insufficient_items("STORE_ACCESS"))
                    .clone();
                let set_item = tos2.borrow().attr("__setitem__", &self.classes)?;

                self.eval_stack.push(tos);
                self.eval_stack.push(tos1);
                self.eval_stack.push(tos2);
                self.eval_stack.push(set_item);
                self.handle_callable_object("__setitem__", 3)?;
            }
            OpCode::LOAD_CONST(n) => {
                self.eval_stack.push(self.constants_pool[n].clone());
            }
            OpCode::LOAD_TRUE => {
                self.eval_stack.push(objref!(Object::Boolean(true)));
            }
            OpCode::LOAD_FALSE => {
                self.eval_stack.push(objref!(Object::Boolean(false)));
            }
            OpCode::LOAD_LOCAL(n) => {
                let local = self.top_frame().get_local(n);
                self.eval_stack.push(local);
            }
            // TODO: GH-10
            OpCode::LOAD_DEREF(_) => todo!(),
            OpCode::LOAD_GLOBAL(n) => {
                let name = self.constants_pool[n].clone();
                let Object::String(ref name) = *name.borrow() else {
                    panic!("Constant object {n} expected to be a string, but is not");
                };

                let global = if let Some(global) = self.globals.get(name) {
                    global
                } else if let Some(global) = self.builtins.get(name) {
                    global
                } else {
                    return Err(RuntimeError::new(&format!(
                        "global name '{name}' is not defined"
                    )));
                }
                .clone();

                self.eval_stack.push(global);
            }
            OpCode::LOAD_ATTR(n) => {
                let tos = self
                    .eval_stack
                    .last()
                    .expect(&insufficient_items("STORE_ACCESS"))
                    .clone();
                let name = self.constants_pool[n].clone();
                let Object::String(ref name) = *name.borrow() else {
                    panic!("Constant object {n} expected to be a string, but is not");
                };

                self.eval_stack
                    .push(tos.borrow().attr(name, &self.classes)?);
            }
            OpCode::LOAD_ACCESS => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("LOAD_ACCESS"));
                let tos1 = self
                    .eval_stack
                    .last()
                    .expect(&insufficient_items("LOAD_ACCESS"))
                    .clone();
                let get_item = tos1.borrow().attr("__getitem__", &self.classes)?;

                self.eval_stack.push(tos);
                self.eval_stack.push(tos1);
                self.eval_stack.push(get_item);
                self.handle_callable_object("__getitem__", 2)?;
            }
            OpCode::MAKE_FUNCTION(n, m) => {
                if !matches!(*self.constants_pool[m].borrow(), Object::Code(_)) {
                    panic!("Constant object {m} expected to be a code object, but is not");
                };

                self.eval_stack
                    .push(objref!(Object::Function(CompiledFunction::new(
                        n,
                        FunctionType::Python(m)
                    ))));
            }
            OpCode::CALL_FUNCTION(n) => {
                // We need to increment the caller frame's IP before handle_callable_object. This way,
                // we don't accidentally increment the IP of the called function's frame if one is created
                // (i.e. it is a python-defined function).
                inc_ip = false;
                self.top_frame().inc_ip(1);

                self.handle_callable_object("__call__", n)?;
            }
            OpCode::BUILD_LIST(n) => {
                let mut new_list = Vec::new();
                for _ in 0..n {
                    let tos = self
                        .eval_stack
                        .pop()
                        .expect(&insufficient_items("BUILD_LIST"));
                    new_list.push(tos);
                }
                self.eval_stack.push(objref!(Object::List(new_list)));
            }
            OpCode::BUILD_DICT(n) => {
                if !n.is_multiple_of(2) {
                    panic!("Cannot build dict with {n} values, it is not even");
                }

                let mut new_dict = Vec::new();
                let mut key = None;
                for _ in 0..n {
                    let tos = self
                        .eval_stack
                        .pop()
                        .expect(&insufficient_items("BUILD_DICT"));
                    if let Some(k) = key {
                        new_dict.push((k, tos));
                        key = None;
                    } else if let Object::String(ref k) = *tos.borrow() {
                        key = Some(k.clone());
                    } else {
                        panic!("PDP does not support building dicts with non-string keys");
                    }
                }
                self.eval_stack.push(objref!(Object::Dict(new_dict)));
            }
            OpCode::BUILD_SET(n) => {
                let mut new_set = Vec::new();
                for _ in 0..n {
                    let tos = self
                        .eval_stack
                        .pop()
                        .expect(&insufficient_items("BUILD_SET"));
                    new_set.push(tos);
                }
                self.eval_stack.push(objref!(Object::Set(new_set)));
            }
            OpCode::RETURN_VALUE => {
                // Function frame is over, and caller frame has already been incremented in CALL_FUNCTION.
                inc_ip = false;

                if let Some(old_frame) = self.frame_stack.pop() {
                    if old_frame.from_generator {
                        // Ignore the "return value" by popping it
                        self.eval_stack.pop();
                        let substack = self.eval_stack.split_off(old_frame.bytecode_offset);

                        let tos = self
                            .eval_stack
                            .last()
                            .expect(&insufficient_items("RETURN_VALUE"))
                            .clone();
                        let Object::Generator(ref mut generator) = *tos.borrow_mut() else {
                            panic!(
                                "TOS must be a generator when returning from a from_generator frame"
                            );
                        };
                        self.eval_stack.push(generator.last_value());
                        generator.finish();
                        generator.set_eval_stack(substack);
                    } else {
                        let retval = self
                            .eval_stack
                            .pop()
                            .expect(&insufficient_items("RETURN_VALUE"));
                        self.eval_stack.truncate(old_frame.bytecode_offset);
                        self.eval_stack.push(retval);
                    }
                } else {
                    panic!("We somehow just returned from a frame that doesn't exist??");
                }
            }
            OpCode::YIELD_VALUE => {
                inc_ip = false;
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("YIELD_VALUE"));

                let frame = self
                    .frame_stack
                    .pop()
                    .expect("We somehow just returned from a frame that doesn't exist??");
                if frame.from_generator {
                    let generator = self
                        .eval_stack
                        .last()
                        .expect(&insufficient_items("YIELD_VALUE"))
                        .clone();
                    let Object::Generator(ref mut generator) = *generator.borrow_mut() else {
                        panic!("TOS1 expected to be a generator, but is not {generator:?}");
                    };
                    let last_value = generator.last_value();

                    // Update generator object
                    generator.set_ip(frame.ip + 1);
                    generator.set_local_vars(frame.local_vars);
                    generator.set_last_value(tos);
                    generator.set_eval_stack(self.eval_stack.split_off(frame.bytecode_offset));

                    self.eval_stack.push(last_value);
                } else {
                    // Create a new generator object
                    self.eval_stack
                        .push(objref!(Object::Generator(FrozenGenerator::new(
                            frame.local_vars,
                            frame.bytecode,
                            frame.ip + 1,
                            tos,
                            false,
                        ))));
                }
            }
            OpCode::PUSH_TEMP => {
                let tos = self
                    .eval_stack
                    .pop()
                    .expect(&insufficient_items("PUSH_TEMP"));
                self.temp_stack.push(tos);
            }
            OpCode::POP_TEMP => {
                let tempval = self
                    .temp_stack
                    .pop()
                    .expect(&insufficient_items("POP_TEMP"));
                self.eval_stack.push(tempval);
            }
        }

        if inc_ip && !self.frame_stack.is_empty() {
            self.top_frame().inc_ip(1);
        }

        Ok(())
    }

    pub fn handle_callable_object(
        &mut self,
        func_name: &str,
        argc: usize,
    ) -> Result<(), RuntimeError> {
        let tos = self
            .eval_stack
            .last()
            .expect(&insufficient_items("handle_callable_object()"))
            .clone();
        let tos_class = tos.borrow().class(&self.classes).name();
        let call = tos
            .borrow()
            .attr("__call__", &self.classes)
            .map_err(|_| RuntimeError::new(&format!("'{tos_class}' object is not callable")))?;

        if let Object::Function(ref func) = *call.borrow() {
            self.execute_function(func_name, func, argc)?;
        } else {
            return Err(RuntimeError::new(&format!(
                "'{tos_class}' object is not callable"
            )));
        }

        Ok(())
    }

    pub fn execute_function(
        &mut self,
        func_name: &str,
        func: &CompiledFunction,
        argc: usize,
    ) -> Result<(), RuntimeError> {
        if self.eval_stack.len() < argc {
            panic!("Not enough values in stack for argc {argc}");
        } else if !func.ignore_argc() && func.argc() != argc {
            return Err(RuntimeError::new(&format!(
                "{func_name}() takes {} positional arguments but {argc} was given",
                func.argc()
            )));
        }

        match func.code() {
            FunctionType::Rust(f) => {
                f(self)?;
            }
            FunctionType::Python(f_idx) => {
                self.called_python_func = true;
                let f_obj = self.constants_pool[*f_idx].clone();
                let args = self.eval_stack.split_off(self.eval_stack.len() - argc);
                if let Object::Code(ref f) = *f_obj.borrow() {
                    self.frame_stack.push(
                        f.as_frame()
                            .with_arguments(args)
                            .with_offset(self.eval_stack.len()),
                    );
                } else {
                    panic!("Constant object {f_idx} expected to be a function, but is not");
                }

                let current_frame_idx = self.frame_stack.len();
                while let Some(frame) = self.frame_stack.get(current_frame_idx) {
                    let bytecode_offset = frame.bytecode_offset;
                    if let Err(e) = self.execute_opcode(frame.next_instruction()) {
                        self.eval_stack.truncate(bytecode_offset);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn handle_generator(&mut self) -> Result<(), RuntimeError> {
        let tos = self
            .eval_stack
            .last()
            .expect(&insufficient_items("handle_generator()"))
            .clone();
        let Object::Generator(ref generator) = *tos.borrow() else {
            panic!("TOS must be a boolean when calling handle_generator()");
        };
        self.frame_stack
            .push(generator.as_frame().with_offset(self.eval_stack.len()));
        self.eval_stack.extend_from_slice(generator.eval_stack());

        Ok(())
    }

    #[inline(always)]
    fn top_frame(&mut self) -> &mut Frame {
        self.frame_stack
            .last_mut()
            .expect("Frame stack is empty before execution has terminated")
    }
}

#[derive(Debug, Default)]
struct Frame {
    bytecode_offset: usize,
    local_vars: Vec<ObjectRef>,
    // TODO: GH-10
    // free_vars: Vec<ObjectRef>,
    // cell_vars: Vec<ObjectRef>,
    bytecode: Vec<OpCode>,
    ip: usize,
    /// When popping this frame, there's a generator at TOS waiting
    from_generator: bool,
}

/// CodeObject -> Frame
impl CodeObject {
    fn as_frame(&self) -> Frame {
        Frame::new(self.bytecode().clone(), self.local_var_num())
    }
}

/// FrozenGenerator -> Frame
impl FrozenGenerator {
    fn as_frame(&self) -> Frame {
        Frame {
            bytecode_offset: 0,
            local_vars: self.local_vars().clone(),
            bytecode: self.bytecode().clone(),
            ip: self.ip(),
            from_generator: true,
        }
    }
}

impl Frame {
    fn new(instructions: Vec<OpCode>, local_var_num: usize) -> Self {
        let mut local_vars = Vec::with_capacity(local_var_num);
        for _ in 0..local_var_num {
            local_vars.push(objref!(Object::None));
        }

        Self {
            bytecode_offset: 0,
            local_vars,
            bytecode: instructions,
            ip: 0,
            from_generator: false,
        }
    }

    pub fn with_arguments(mut self, args: Vec<ObjectRef>) -> Self {
        for (i, arg) in args.iter().rev().enumerate() {
            self.local_vars[i] = arg.clone();
        }
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.bytecode_offset = offset;
        self
    }

    pub fn next_instruction(&self) -> OpCode {
        self.bytecode[self.ip]
    }

    pub fn set_ip(&mut self, n: usize) {
        if n >= self.bytecode.len() {
            panic!("IP set beyond its limits");
        }
        self.ip = n;
    }

    pub fn inc_ip(&mut self, n: usize) {
        self.ip += n;
        if self.ip >= self.bytecode.len() {
            panic!("IP incremented beyond its limits");
        }
    }

    pub fn get_local(&self, local_idx: usize) -> ObjectRef {
        self.local_vars[local_idx].clone()
    }

    pub fn set_local(&mut self, local_idx: usize, new_value: ObjectRef) {
        self.local_vars[local_idx] = new_value;
    }
}
