use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::AddAssign;
use std::rc::Rc;

use ordered_float::OrderedFloat;

use super::{OpCode, objects::*};
use crate::bytecode::objects::Object;
use crate::parser::ptag::{AstNode, OperationTree};
use crate::parser::{building_blocks::*, markers::*, symbol_table::SymbolTable};
use crate::{non_identity_ast, objref};

use log::debug;

#[inline(always)]
fn digits(n: usize) -> usize {
    (n as f64).log10().floor() as usize
}

fn display_constants(constants: &[ObjectRef], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for constant in constants {
        let borrow = constant.borrow();

        if let Object::Code(co) = &*borrow {
            display_constants(co.constants_pool(), f)?;
            writeln!(f, "{:p}:", &*borrow)?;
            display_bytecode(co.bytecode(), co.constants_pool(), f)?;
        }
    }

    Ok(())
}

fn display_bytecode(
    instructions: &[OpCode],
    constants_pool: &[ObjectRef],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let total_spaces = digits(instructions.len()) + 5;

    for (instruction_idx, instruction) in instructions.iter().enumerate() {
        let idx_spaces = digits(instruction_idx);

        write!(
            f,
            "{instruction_idx}{}",
            " ".repeat(total_spaces - idx_spaces)
        )?;
        match instruction {
            OpCode::NOP => write!(f, "NOP")?,
            OpCode::POP_TOP => write!(f, "POP_TOP")?,
            OpCode::SWAP_TOP => write!(f, "SWAP_TOP")?,
            OpCode::DUP_TOP => write!(f, "DUP_TOP")?,
            OpCode::JUMP_FORWARD(n) => write!(f, "JUMP_FORWARD {n}")?,
            OpCode::JUMP_IF_FALSE(n) => write!(f, "JUMP_IF_FALSE {n}")?,
            OpCode::JUMP_IF_TRUE(n) => write!(f, "JUMP_IF_TRUE {n}")?,
            OpCode::JUMP_ABSOLUTE(n) => write!(f, "JUMP_ABSOLUTE {n}")?,
            OpCode::MAKE_GENERATOR => write!(f, "MAKE_GENERATOR")?,
            OpCode::FOR_ITER(n) => write!(f, "FOR_ITER {n}")?,
            OpCode::STORE_LOCAL(n) => write!(f, "STORE_LOCAL {n}")?,
            OpCode::STORE_DEREF(n) => write!(f, "STORE_DEREF {n}")?,
            OpCode::STORE_GLOBAL(n) => {
                let name = constants_pool
                    .get(*n)
                    .expect(&format!("Constant {n} should exist"));
                let Object::String(name) = &*name.borrow() else {
                    panic!("Constant {n} should be a string");
                };
                write!(f, "STORE_GLOBAL '{name}'")?
            }
            OpCode::STORE_ATTR(n) => {
                let attr = constants_pool
                    .get(*n)
                    .expect(&format!("Constant {n} should exist"));
                let Object::String(attr) = &*attr.borrow() else {
                    panic!("Constant {n} should be a string");
                };
                write!(f, "STORE_ATTR '{attr}'")?
            }
            OpCode::STORE_ACCESS => write!(f, "STORE_ACCESS")?,
            OpCode::LOAD_CONST(n) => {
                let c = constants_pool
                    .get(*n)
                    .expect(&format!("Constant {n} should exist"));
                let c_display = match &*c.borrow() {
                    Object::None => "None".to_string(),
                    Object::Number(num) => format!("{num}"),
                    Object::Boolean(b) => (if *b { "True" } else { "False" }).to_string(),
                    Object::String(s) => format!("'{s}'"),
                    Object::Code(code_object) => format!("Code({code_object:p})"),
                    _ => panic!("This constant is a non-const type"),
                };
                write!(f, "LOAD_CONST {c_display}")?
            }
            OpCode::LOAD_TRUE => write!(f, "LOAD_TRUE")?,
            OpCode::LOAD_FALSE => write!(f, "LOAD_FALSE")?,
            OpCode::LOAD_LOCAL(n) => write!(f, "LOAD_LOCAL {n}")?,
            OpCode::LOAD_DEREF(n) => write!(f, "LOAD_DEREF {n}")?,
            OpCode::LOAD_GLOBAL(n) => {
                let name = constants_pool
                    .get(*n)
                    .expect(&format!("Constant {n} should exist"));
                let Object::String(name) = &*name.borrow() else {
                    panic!("Constant {n} should be a string");
                };
                write!(f, "LOAD_GLOBAL '{name}'")?
            }
            OpCode::LOAD_ATTR(n) => {
                let attr = constants_pool
                    .get(*n)
                    .expect(&format!("Constant {n} should exist"));
                let Object::String(attr) = &*attr.borrow() else {
                    panic!("Constant {n} should be a string");
                };
                write!(f, "LOAD_ATTR '{attr}'")?
            }
            OpCode::LOAD_ACCESS => write!(f, "LOAD_ACCESS")?,
            OpCode::MAKE_FUNCTION(n) => write!(f, "MAKE_FUNCTION {n}")?,
            OpCode::CALL_FUNCTION(n) => write!(f, "CALL_FUNCTION {n}")?,
            OpCode::BUILD_LIST(n) => write!(f, "BUILD_LIST {n}")?,
            OpCode::BUILD_DICT(n) => write!(f, "BUILD_DICT {n}")?,
            OpCode::BUILD_SET(n) => write!(f, "BUILD_SET {n}")?,
            OpCode::RETURN_VALUE => write!(f, "RETURN_VALUE")?,
            OpCode::PUSH_TEMP => write!(f, "PUSH_TEMP")?,
            OpCode::POP_TEMP => write!(f, "POP_TEMP")?,
        }
        writeln!(f)?;
    }
    writeln!(f)
}

#[derive(Debug, Default, Clone, Copy)]
struct Emissions(usize);

impl AddAssign for Emissions {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ConstIndex(usize);

#[derive(Debug)]
struct LoopContext {
    start: usize,
    break_points: Vec<usize>,
}

#[derive(Debug)]
pub struct BytecodeEmitter {
    is_emitted: bool,
    symbols: SymbolTable,
    compiled_child_symbol_tables: usize,
    constants_pool: Vec<ObjectRef>,
    string_literal_const_idx: HashMap<String, usize>,
    num_literal_const_idx: HashMap<OrderedFloat<f64>, usize>,
    loop_contexts: Vec<LoopContext>,
    instructions: Vec<OpCode>,
}

impl Display for BytecodeEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_emitted {
            return write!(f, "Nothing to display, bytecode has not been emitted yet.");
        }

        display_constants(&self.constants_pool, f)?;
        writeln!(f, "<module>:")?;
        display_bytecode(&self.instructions, &self.constants_pool, f)
    }
}

impl BytecodeEmitter {
    pub fn new(symbols: SymbolTable) -> Self {
        Self {
            is_emitted: false,
            symbols,
            compiled_child_symbol_tables: 0,
            constants_pool: vec![objref!(Object::None)],
            string_literal_const_idx: HashMap::new(),
            num_literal_const_idx: HashMap::new(),
            loop_contexts: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn emit(&mut self, ast: &MarkedAstNode) {
        self.ast(ast);
        // If instructions don't already end with RETURN_VALUE, add returning None.
        if !self
            .instructions
            .last()
            .is_some_and(|i| matches!(i, OpCode::RETURN_VALUE))
        {
            self.instructions.push(OpCode::LOAD_CONST(0));
            self.instructions.push(OpCode::RETURN_VALUE);
        }

        self.is_emitted = true;
    }

    fn ast(&mut self, ast: &MarkedAstNode) -> Emissions {
        debug!("BytecodeEmitter::ast() started");

        let total = match &ast.comp {
            AstNode::block(code_units) => self.block(code_units),
            AstNode::empty => Emissions(0),
            AstNode::if_stmt { condition, then } => self.if_stmt(condition, then),
            AstNode::while_loop { condition, body } => self.while_loop(condition, body),
            AstNode::for_loop {
                loop_variable,
                iterator,
                body,
            } => self.for_loop(loop_variable, iterator, body),
            AstNode::r#continue => self.r#continue(),
            AstNode::r#break => self.r#break(),
            AstNode::return_stmt(value) => self.return_stmt(value),
            AstNode::function_def {
                identifier,
                parameters,
                body,
            } => self.function_def(identifier, parameters, body),
            AstNode::function_call {
                function,
                arguments,
            } => self.function_call(function, arguments),
            AstNode::assign_op {
                variable,
                accesses,
                asop,
                value,
            } => self.assign_op(variable, accesses, asop, value),
            bad => panic!("Tried using BytecodeEmitter::ast() on {bad:?}"),
        };

        debug!("BytecodeEmitter::ast() ended");
        total
    }

    /// ```
    /// Unit 1
    /// Unit 2
    /// ...
    /// Unit N-1
    /// Unit N
    /// ```
    fn block(&mut self, code_units: &Vec<MarkedAstNode>) -> Emissions {
        debug!("BytecodeEmitter::block() started");
        let mut total = Emissions(0);

        for unit in code_units {
            total += self.ast(unit);
        }

        debug!("BytecodeEmitter::block() ended");
        total
    }

    /// ```
    /// Condition
    /// JUMP_IF_FALSE
    /// Then
    /// ```
    fn if_stmt(&mut self, condition: &MarkedOperationTree, then: &MarkedAstNode) -> Emissions {
        debug!("BytecodeEmitter::if_stmt() started");
        let mut total = Emissions(0);

        total += self.operation_tree(condition);
        let jump_ip = self.instructions.len();
        self.instructions.push(OpCode::NOP);
        total.0 += 1;
        let then_size = self.ast(then);
        total += then_size;

        // Replace NOP with JUMP_IF_FALSE
        *self
            .instructions
            .get_mut(jump_ip)
            .expect("Instruction wasn't found") = OpCode::JUMP_IF_FALSE(then_size.0 + 1);

        debug!("BytecodeEmitter::if_stmt() ended");
        total
    }

    /// ```
    /// Condition
    /// JUMP_IF_FALSE
    /// body
    /// JUMP_ABSOLUTE
    /// ```
    fn while_loop(&mut self, condition: &MarkedOperationTree, body: &MarkedAstNode) -> Emissions {
        debug!("BytecodeEmitter::while_loop() started");
        let mut total = Emissions(0);
        self.loop_contexts.push(LoopContext {
            start: self.instructions.len(),
            break_points: Vec::new(),
        });

        total += self.operation_tree(condition);
        let jump_ip = self.instructions.len();
        self.instructions.push(OpCode::NOP);
        total.0 += 1;
        let body_size = self.ast(body);
        total += body_size;
        self.instructions.push(OpCode::JUMP_ABSOLUTE(jump_ip));
        total.0 += 1;
        let loop_end = self.instructions.len();

        // Replace NOP with JUMP_IF_FALSE
        *self
            .instructions
            .get_mut(jump_ip)
            .expect("Instruction wasn't found") = OpCode::JUMP_IF_FALSE(body_size.0 + 2);
        // Replace all break NOPs with JUMP_ABSOLUTE
        for br in self
            .loop_contexts
            .pop()
            .expect("Loop context was not set")
            .break_points
        {
            *self.instructions.get_mut(br).expect("Break wasn't found") =
                OpCode::JUMP_ABSOLUTE(loop_end);
        }

        debug!("BytecodeEmitter::while_loop() ended");
        total
    }

    /// ```
    /// Iterator
    /// BUILD_GENERATOR
    /// FOR_ITER
    /// STORE_{LOCAL|DEREF|GLOBAL}
    /// Body
    /// JUMP_ABSOLUTE
    /// ```
    fn for_loop(
        &mut self,
        loop_variable: &MarkedString,
        iterator: &MarkedOperationTree,
        body: &MarkedAstNode,
    ) -> Emissions {
        debug!("BytecodeEmitter::for_loop() started");
        let mut total = Emissions(0);

        total += self.operation_tree(iterator);
        self.instructions.push(OpCode::MAKE_GENERATOR);
        total.0 += 1;
        let loop_ip = self.instructions.len();
        self.loop_contexts.push(LoopContext {
            start: loop_ip,
            break_points: Vec::new(),
        });
        self.instructions.push(OpCode::NOP);
        total.0 += 1;
        total += self.emit_store(loop_variable);
        let body_size = self.ast(body);
        total += body_size;
        self.instructions.push(OpCode::JUMP_ABSOLUTE(loop_ip));
        total.0 += 1;
        let loop_end = self.instructions.len();

        // Replace NOP with FOR_ITER
        *self
            .instructions
            .get_mut(loop_ip)
            .expect("Instruction wasn't found") = OpCode::FOR_ITER(body_size.0 + 3);
        // Replace all break NOPs with JUMP_ABSOLUTE
        for br in self
            .loop_contexts
            .pop()
            .expect("Loop context was not set")
            .break_points
        {
            *self.instructions.get_mut(br).expect("Break wasn't found") =
                OpCode::JUMP_ABSOLUTE(loop_end);
        }

        debug!("BytecodeEmitter::for_loop() ended");
        total
    }

    /// ```
    /// JUMP_ABSOLUTE
    /// ```
    fn r#continue(&mut self) -> Emissions {
        debug!("BytecodeEmitter::continue() started");
        let mut total = Emissions(0);

        let target = self
            .loop_contexts
            .last()
            .expect("Loop context is missing")
            .start;
        self.instructions.push(OpCode::JUMP_ABSOLUTE(target));
        total.0 += 1;

        debug!("BytecodeEmitter::continue() ended");
        total
    }

    /// ```
    /// NOP -> JUMP_ABSOLUTE
    /// ```
    fn r#break(&mut self) -> Emissions {
        debug!("BytecodeEmitter::break() started");
        let mut total = Emissions(0);

        self.loop_contexts
            .last_mut()
            .expect("Loop context is missing")
            .break_points
            .push(self.instructions.len());
        // Will be replaced with JUMP_ABSOLUTE once the IP is known
        self.instructions.push(OpCode::NOP);
        total.0 += 1;

        debug!("BytecodeEmitter::break() ended");
        total
    }

    /// ```
    /// {Value|LOAD_CONST}
    /// RETURN_VALUE
    /// ```
    fn return_stmt(&mut self, value: &Option<Box<MarkedOperationTree>>) -> Emissions {
        debug!("BytecodeEmitter::return_stmt() started");
        let mut total = Emissions(0);

        total.0 += match value {
            Some(val) => self.operation_tree(val).0,
            None => {
                self.instructions.push(OpCode::LOAD_CONST(0));
                1
            }
        };
        self.instructions.push(OpCode::RETURN_VALUE);
        total.0 += 1;

        debug!("BytecodeEmitter::return_stmt() ended");
        total
    }

    /// ```
    /// LOAD_CONST
    /// MAKE_FUNCTION
    /// STORE_{LOCAL|DEREF|GLOBAL}
    /// ```
    fn function_def(
        &mut self,
        identifier: &MarkedString,
        parameters: &[MarkedString],
        body: &MarkedAstNode,
    ) -> Emissions {
        debug!("BytecodeEmitter::function_def() started");
        let mut total = Emissions(0);

        // Build code object of function and add it to constants pool
        let child_symbols = self.symbols.child(self.compiled_child_symbol_tables);
        let mut function_emitter = Self::new(child_symbols.clone());
        function_emitter.emit(body);
        let (child_instructions, _, child_constants) = function_emitter.dissolve();
        let code_object = CodeObject::new(
            child_symbols.num_local_vars(),
            child_symbols.num_deref_vars(),
            child_constants,
            child_instructions,
        );
        let code_object_idx = self.constants_pool.len();
        self.constants_pool.push(objref!(Object::Code(code_object)));

        // Actual bytecode emission
        self.instructions.push(OpCode::LOAD_CONST(code_object_idx));
        total.0 += 1;
        self.instructions
            .push(OpCode::MAKE_FUNCTION(parameters.len()));
        total.0 += 1;
        total += self.emit_store(identifier);

        debug!("BytecodeEmitter::function_def() ended");
        total
    }

    /// ```
    /// Arguments
    /// LOAD_{LOCAL|DEREF|GLOBAL}
    /// CALL_FUNCTION
    /// ```
    fn function_call(
        &mut self,
        function: &MarkedString,
        arguments: &[MarkedOperationTree],
    ) -> Emissions {
        debug!("BytecodeEmitter::function_call() started");
        let mut total = Emissions(0);

        for arg in arguments.iter().rev() {
            total += self.operation_tree(arg);
        }
        total += self.emit_load(function);
        self.instructions
            .push(OpCode::CALL_FUNCTION(arguments.len()));
        total.0 += 1;

        debug!("BytecodeEmitter::function_call() ended");
        total
    }

    /// ```
    /// [if there are accesses
    ///     LOAD_{LOCAL|DEREF|GLOBAL}
    ///     *all except last access
    ///         Access
    ///         LOAD_ACCESS
    ///         SWAP_TOP
    ///         POP_TOP
    ///     *
    ///     Last access
    ///     [if not pure assign
    ///         DUP_TOP
    ///         PUSH_TEMP
    ///         LOAD_ACCESS
    ///         Value
    ///         SWAP_TOP
    ///         LOAD_ATTR
    ///         CALL_FUNCTION
    ///         POP_TEMP
    ///         SWAP_TOP
    ///     ][else
    ///         Value
    ///     ]
    ///     STORE_ACCESS
    ///     POP_TOP
    /// ][else
    ///     Value
    ///     STORE_{LOCAL|DEREF|GLOBAL}
    /// ]
    /// ```
    fn assign_op(
        &mut self,
        variable: &MarkedString,
        accesses: &[MarkedOperationTree],
        asop: &MarkedAsop,
        value: &MarkedOperationTree,
    ) -> Emissions {
        debug!("BytecodeEmitter::assign_op() started");
        let mut total = Emissions(0);

        if accesses.is_empty() {
            total += self.operation_tree(value);
            total += self.emit_store(variable);
        } else {
            total += self.emit_load(variable);
            for access in &accesses[..accesses.len() - 1] {
                total += self.operation_tree(access);
                self.instructions.push(OpCode::LOAD_ACCESS);
                total.0 += 1;
                self.instructions.push(OpCode::SWAP_TOP);
                total.0 += 1;
                self.instructions.push(OpCode::POP_TOP);
                total.0 += 1;
            }
            // Unwrap is safe because we already checked that `accesses` is not empty
            #[allow(clippy::unwrap_used)]
            let last_access = accesses.last().unwrap();
            total += self.operation_tree(last_access);

            if matches!(asop.comp, Asop::Assign) {
                total += self.operation_tree(value);
            } else {
                self.instructions.push(OpCode::DUP_TOP);
                total.0 += 1;
                self.instructions.push(OpCode::PUSH_TEMP);
                total.0 += 1;
                self.instructions.push(OpCode::LOAD_ACCESS);
                total.0 += 1;
                total += self.operation_tree(value);
                self.instructions.push(OpCode::SWAP_TOP);
                total.0 += 1;
                let op_method_idx =
                    self.const_string(&asop.comp.dunderscore_method().to_string().into());
                self.instructions.push(OpCode::LOAD_ATTR(op_method_idx.0));
                total.0 += 1;
                self.instructions.push(OpCode::CALL_FUNCTION(1));
                total.0 += 1;
                self.instructions.push(OpCode::POP_TEMP);
                total.0 += 1;
                self.instructions.push(OpCode::SWAP_TOP);
                total.0 += 1;
            }

            self.instructions.push(OpCode::STORE_ACCESS);
            total.0 += 1;
            self.instructions.push(OpCode::POP_TOP);
            total.0 += 1;
        }

        debug!("BytecodeEmitter::assign_op() ended");
        total
    }

    fn operation_tree(&mut self, op_tree: &MarkedOperationTree) -> Emissions {
        debug!("BytecodeEmitter::operation_tree() started");
        let mut total = Emissions(0);

        match &op_tree.comp {
            OperationTree::Unary { operation, value } => {
                total += self.operation_tree(value);
                let op_method_idx = self
                    .const_string(&operation.comp.dunderscore_method_unary().to_string().into());
                self.instructions.push(OpCode::LOAD_ATTR(op_method_idx.0));
                total.0 += 1;
                self.instructions.push(OpCode::CALL_FUNCTION(0));
                total.0 += 1;
            }
            OperationTree::Binary {
                operation,
                left,
                right,
            } => {
                total += self.operation_tree(left);
                let op_method_idx =
                    self.const_string(&operation.comp.dunderscore_method().to_string().into());
                self.instructions.push(OpCode::LOAD_ATTR(op_method_idx.0));
                total.0 += 1;
                total += self.operation_tree(right);
                self.instructions.push(OpCode::SWAP_TOP);
                total.0 += 1;
                self.instructions.push(OpCode::CALL_FUNCTION(1));
                total.0 += 1;
            }
            OperationTree::Identity(marked_component) => match &marked_component.comp {
                AstNode::function_call {
                    function,
                    arguments,
                } => {
                    total += self.function_call(function, arguments);
                }
                AstNode::variable {
                    identifier,
                    accesses,
                } => {
                    total += self.emit_load(identifier);
                    for access in accesses {
                        total += self.operation_tree(access);
                        self.instructions.push(OpCode::LOAD_ACCESS);
                        total.0 += 1;
                    }
                }
                AstNode::list(list) => {
                    for item in list.iter().rev() {
                        total += self.operation_tree(item);
                    }
                    self.instructions.push(OpCode::BUILD_LIST(list.len()));
                    total.0 += 1;
                }
                AstNode::dictionary(dictionary) => {
                    for (key, value) in dictionary.iter().rev() {
                        let key_idx = self.const_string(key);
                        total += self.operation_tree(value);
                        self.instructions.push(OpCode::LOAD_CONST(key_idx.0));
                        total.0 += 1;
                    }
                    self.instructions.push(OpCode::BUILD_DICT(dictionary.len()));
                    total.0 += 1;
                }
                AstNode::set(set) => {
                    for item in set.iter().rev() {
                        total += self.operation_tree(item);
                    }
                    self.instructions.push(OpCode::BUILD_SET(set.len()));
                    total.0 += 1;
                }
                AstNode::string(s) => {
                    let string_idx = self.const_string(s);
                    self.instructions.push(OpCode::LOAD_CONST(string_idx.0));
                    total.0 += 1;
                }
                AstNode::number(n) => {
                    let number_idx = self.const_num(n);
                    self.instructions.push(OpCode::LOAD_CONST(number_idx.0));
                    total.0 += 1;
                }
                AstNode::boolean(b) => {
                    self.instructions.push(if b.comp {
                        OpCode::LOAD_TRUE
                    } else {
                        OpCode::LOAD_FALSE
                    });
                }
                non_identity_ast!() => {
                    panic!("Tried calling operation_tree() with {marked_component:?}");
                }
            },
        }

        debug!("BytecodeEmitter::operation_tree() ended");
        total
    }

    fn emit_store(&mut self, name: &MarkedString) -> Emissions {
        if let Some(idx) = self.symbols.local_idx(name) {
            self.instructions.push(OpCode::STORE_LOCAL(idx));
        } else if let Some(idx) = self.symbols.deref_idx(name) {
            self.instructions.push(OpCode::STORE_DEREF(idx));
        } else {
            let name_idx = self.const_string(name).0;
            self.instructions.push(OpCode::STORE_GLOBAL(name_idx));
        }

        Emissions(1)
    }

    fn emit_load(&mut self, name: &MarkedString) -> Emissions {
        if let Some(idx) = self.symbols.local_idx(name) {
            self.instructions.push(OpCode::LOAD_LOCAL(idx));
        } else if let Some(idx) = self.symbols.deref_idx(name) {
            self.instructions.push(OpCode::LOAD_DEREF(idx));
        } else {
            let name_idx = self.const_string(name).0;
            self.instructions.push(OpCode::LOAD_GLOBAL(name_idx));
        }

        Emissions(1)
    }

    fn const_string(&mut self, s: &MarkedString) -> ConstIndex {
        match self.string_literal_const_idx.get(&s.comp) {
            Some(idx) => ConstIndex(*idx),
            None => {
                let idx = self.constants_pool.len();
                self.constants_pool
                    .push(objref!(Object::String(s.comp.clone())));
                self.string_literal_const_idx.insert(s.comp.clone(), idx);
                ConstIndex(idx)
            }
        }
    }

    fn const_num(&mut self, n: &MarkedNumber) -> ConstIndex {
        match self.num_literal_const_idx.get(&n.comp.into()) {
            Some(idx) => ConstIndex(*idx),
            None => {
                let idx = self.constants_pool.len();
                self.constants_pool.push(objref!(Object::Number(n.comp)));
                self.num_literal_const_idx.insert(n.comp.into(), idx);
                ConstIndex(idx)
            }
        }
    }

    /// Consumes the emitter and returns its instructions, symbol_table, and constants_pool respectively.
    pub fn dissolve(self) -> (Vec<OpCode>, SymbolTable, Vec<ObjectRef>) {
        let BytecodeEmitter {
            instructions,
            symbols,
            constants_pool,
            ..
        } = self;
        (instructions, symbols, constants_pool)
    }
}
