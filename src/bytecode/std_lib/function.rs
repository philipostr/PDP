use std::rc::Rc;

use crate::{
    bytecode::{
        VM,
        objects::{Class, Object},
        vm::RuntimeError,
    },
    class_method, objref,
};

pub fn init_class() -> Class {
    let mut class = Class::new("Function");

    class_method!(class, __call__);
    class_method!(class, __bool__, 1);
    class_method!(class, __eq__, 2);

    class
}

fn __call__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Function(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.execute_function("__call__", slf, slf.argc())?;

    Ok(())
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    vm.pop_tos();
    vm.push_tos(objref!(Object::Boolean(true)));

    Ok(())
}

fn __eq__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Function(_) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Function(_) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Function' == '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(Rc::ptr_eq(&slf_, &other_))));

    Ok(())
}
