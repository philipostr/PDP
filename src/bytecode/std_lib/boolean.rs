use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::VM;
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("Boolean");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __neg__, 1);
    class_method!(class, __eq__, 2);
    class_method!(class, __inv__, 1);
    class_method!(class, __lt__, 2);
    class_method!(class, __le__, 2);
    class_method!(class, __gt__, 2);
    class_method!(class, __ge__, 2);

    class
}

fn __bool__(_vm: &mut VM) -> Result<(), RuntimeError> {
    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::String(
        if slf { "True" } else { "False" }.to_string()
    )));

    Ok(())
}

fn __neg__(vm: &mut VM) -> Result<(), RuntimeError> {
    __inv__(vm)
}

fn __eq__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Boolean(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Boolean' == '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf == other)));

    Ok(())
}

fn __inv__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    vm.push_tos(objref!(Object::Boolean(!slf)));

    Ok(())
}

fn __lt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Boolean(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Boolean' < '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(!slf && other)));

    Ok(())
}

fn __le__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Boolean(_other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Boolean' <= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(!slf)));

    Ok(())
}

fn __gt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Boolean(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Boolean' > '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf && !other)));

    Ok(())
}

fn __ge__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Boolean(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Boolean(_other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Boolean' >= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf)));

    Ok(())
}
