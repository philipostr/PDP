use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::VM;
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("String");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __eq__, 2);
    class_method!(class, __lt__, 2);
    class_method!(class, __le__, 2);
    class_method!(class, __gt__, 2);
    class_method!(class, __ge__, 2);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(!slf.is_empty())));

    Ok(())
}

fn __str__(_vm: &mut VM) -> Result<(), RuntimeError> {
    Ok(())
}

fn __eq__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::String(ref other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'String' == '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf == other)));

    Ok(())
}

fn __lt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::String(ref other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'String' < '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf < other)));

    Ok(())
}

fn __le__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::String(ref other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'String' <= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf <= other)));

    Ok(())
}

fn __gt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::String(ref other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'String' > '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf > other)));

    Ok(())
}

fn __ge__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::String(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::String(ref other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'String' >= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf >= other)));

    Ok(())
}
