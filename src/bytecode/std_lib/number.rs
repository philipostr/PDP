use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::VM;
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("Number");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __add__, 2);
    class_method!(class, __sub__, 2);
    class_method!(class, __mul__, 2);
    class_method!(class, __truediv__, 2);
    class_method!(class, __mod__, 2);
    class_method!(class, __floordiv__, 2);
    class_method!(class, __pow__, 2);
    class_method!(class, __neg__, 1);
    class_method!(class, __eq__, 2);
    class_method!(class, __lt__, 2);
    class_method!(class, __le__, 2);
    class_method!(class, __gt__, 2);
    class_method!(class, __ge__, 2);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(slf != 0.0)));
    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::String(slf.to_string())));

    Ok(())
}

fn __add__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' + '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf + other)));

    Ok(())
}

fn __sub__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' - '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf - other)));

    Ok(())
}

fn __mul__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' * '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf * other)));

    Ok(())
}

fn __truediv__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' / '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf / other)));

    Ok(())
}

fn __mod__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' % '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf % other)));

    Ok(())
}

fn __floordiv__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' // '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number((slf / other).floor())));

    Ok(())
}

fn __pow__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' ** '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Number(slf.powf(other))));

    Ok(())
}

fn __neg__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    vm.push_tos(objref!(Object::Number(-slf)));

    Ok(())
}

fn __eq__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' == '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf == other)));

    Ok(())
}

fn __lt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' < '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf < other)));

    Ok(())
}

fn __le__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' <= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf <= other)));

    Ok(())
}

fn __gt__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' > '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf > other)));

    Ok(())
}

fn __ge__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Number(slf) = *slf_.borrow() else {
        panic!();
    };

    let other_ = vm.pop_tos();
    let Object::Number(other) = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'Number' >= '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(slf >= other)));

    Ok(())
}
