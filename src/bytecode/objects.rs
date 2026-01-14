use crate::bytecode::OpCode;

use std::cell::RefCell;
use std::rc::Rc;

pub type ObjectRef = Rc<RefCell<Object>>;

#[macro_export]
macro_rules! objref {
    ($object:expr) => {
        Rc::new(RefCell::new($object))
    };
}

#[derive(Debug)]
pub enum Object {
    None,
    Number(f64),
    Boolean(bool),
    String(String),
    List,
    Set,
    Dict,
    Code(CodeObject),
    Function,
    Generator,
    Class,
}

#[derive(Debug)]
pub struct CodeObject {
    local_vars_num: usize,
    deref_vars_num: usize,
    constants_pool: Vec<ObjectRef>,
    bytecode: Vec<OpCode>,
}

impl CodeObject {
    pub fn new(
        local_vars_num: usize,
        deref_vars_num: usize,
        constants_pool: Vec<ObjectRef>,
        bytecode: Vec<OpCode>,
    ) -> Self {
        Self {
            local_vars_num,
            deref_vars_num,
            constants_pool,
            bytecode,
        }
    }

    pub fn constants_pool(&self) -> &Vec<ObjectRef> {
        &self.constants_pool
    }

    pub fn bytecode(&self) -> &Vec<OpCode> {
        &self.bytecode
    }
}
