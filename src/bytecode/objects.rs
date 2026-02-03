use super::vm::RuntimeError;
use crate::bytecode::{OpCode, VM};
use crate::util::Map;

use std::cell::RefCell;
use std::rc::Rc;

pub type ObjectRef = Rc<RefCell<Object>>;

#[macro_export]
macro_rules! objref {
    ($object:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($object))
    };
}

#[derive(Debug)]
pub enum Object {
    None,
    Number(f64),
    Boolean(bool),
    String(String),
    List(Vec<ObjectRef>),
    Set(Vec<ObjectRef>),
    Dict(Vec<(String, ObjectRef)>),
    Code(CodeObject),
    Function(CompiledFunction),
    Generator(FrozenGenerator),
    // TODO: GH-9
    // Class,
}

impl Object {
    pub fn class_idx(&self) -> usize {
        // Must be kept updated in VM::start()
        match self {
            Object::None => 0,
            Object::Number(_) => 1,
            Object::Boolean(_) => 2,
            Object::String(_) => 3,
            Object::List(_) => 4,
            Object::Set(_) => 5,
            Object::Dict(_) => 6,
            Object::Code(_) => 7,
            Object::Function(_) => 8,
            Object::Generator(_) => 9,
        }
    }

    pub fn class<'vm>(&self, classes: &'vm [Class]) -> &'vm Class {
        &classes[self.class_idx()]
    }

    pub fn attr(&self, attr: &str, classes: &[Class]) -> Result<ObjectRef, RuntimeError> {
        // TODO: GH-9
        // if let Object::Class(instance) = self {
        //     classes[instance.class()]
        // } else {
        //     ...
        // }.attr(attr)

        self.class(classes).attr(attr)
    }
}

#[derive(Clone, Debug)]
pub struct CodeObject {
    local_vars_num: usize,
    deref_vars_num: usize,
    bytecode: Vec<OpCode>,
}

impl CodeObject {
    pub fn new(local_vars_num: usize, deref_vars_num: usize, bytecode: Vec<OpCode>) -> Self {
        Self {
            local_vars_num,
            deref_vars_num,
            bytecode,
        }
    }

    pub fn bytecode(&self) -> &Vec<OpCode> {
        &self.bytecode
    }

    pub fn local_var_num(&self) -> usize {
        self.local_vars_num
    }
}

#[derive(Debug)]
pub struct CompiledFunction {
    argc: usize,
    /// Only true for builtin Funcion.__call__() class method
    ignore_argc: bool,
    code: FunctionType,
}

impl CompiledFunction {
    pub fn new(argc: usize, code: FunctionType) -> Self {
        Self {
            argc,
            ignore_argc: false,
            code,
        }
    }

    pub fn without_argc(mut self) -> Self {
        self.ignore_argc = true;
        self
    }

    pub fn ignore_argc(&self) -> bool {
        self.ignore_argc
    }

    pub fn argc(&self) -> usize {
        self.argc
    }

    pub fn code(&self) -> &FunctionType {
        &self.code
    }
}

#[derive(Debug)]
pub enum FunctionType {
    Rust(fn(&mut VM) -> Result<(), RuntimeError>),
    /// Holds the index of the code object in the VMs constants pool
    Python(usize),
}

#[derive(Debug)]
pub struct FrozenGenerator {
    local_vars: Vec<ObjectRef>,
    eval_stack: Vec<ObjectRef>,
    // TODO: GH-10
    // free_vars: Vec<ObjectRef>,
    // cell_vars: Vec<ObjectRef>,
    bytecode: Vec<OpCode>,
    ip: usize,
    last_value: ObjectRef,
    is_done: bool,
}

impl FrozenGenerator {
    pub fn new(
        local_vars: Vec<ObjectRef>,
        bytecode: Vec<OpCode>,
        ip: usize,
        initial_value: ObjectRef,
        is_done: bool,
    ) -> Self {
        Self {
            local_vars,
            eval_stack: Vec::new(),
            bytecode,
            ip,
            last_value: initial_value,
            is_done,
        }
    }

    pub fn local_vars(&self) -> &Vec<ObjectRef> {
        &self.local_vars
    }

    pub fn eval_stack(&self) -> &Vec<ObjectRef> {
        &self.eval_stack
    }

    pub fn set_eval_stack(&mut self, eval_stack: Vec<ObjectRef>) {
        self.eval_stack = eval_stack;
    }

    pub fn set_local_vars(&mut self, locals: Vec<ObjectRef>) {
        self.local_vars = locals;
    }

    pub fn bytecode(&self) -> &Vec<OpCode> {
        &self.bytecode
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn is_done(&self) -> bool {
        self.is_done
    }

    pub fn finish(&mut self) {
        self.is_done = true;
    }

    pub fn last_value(&self) -> ObjectRef {
        self.last_value.clone()
    }

    pub fn set_last_value(&mut self, value: ObjectRef) {
        self.last_value = value;
    }
}

#[derive(Debug, Default)]
pub struct Class {
    name: String,
    attrs: Map<ObjectRef>,
}

impl Class {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            attrs: Map::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn attr(&self, attr: &str) -> Result<ObjectRef, RuntimeError> {
        self.attrs
            .get(attr)
            .cloned()
            .ok_or(RuntimeError::new(&format!(
                "'{}' object has no attribute '{attr}'",
                self.name
            )))
    }

    pub fn add_attr(&mut self, attr: &str, val: ObjectRef) {
        self.attrs.insert(attr.to_string(), val);
    }
}
