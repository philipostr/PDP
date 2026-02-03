use crate::{
    bytecode::{
        VM,
        objects::{CompiledFunction, FunctionType, Object, ObjectRef},
        vm::RuntimeError,
    },
    objref,
};

pub mod boolean;
pub mod code;
pub mod dict;
pub mod function;
pub mod generator;
pub mod list;
pub mod none;
pub mod number;
pub mod set;
pub mod string;

#[macro_export]
macro_rules! class_method {
    ($class:ident, $attr:ident) => {
        $class.add_attr(
            stringify!($attr),
            $crate::objref!($crate::bytecode::objects::Object::Function(
                $crate::bytecode::objects::CompiledFunction::new(
                    0,
                    $crate::bytecode::objects::FunctionType::Rust($attr)
                )
                .without_argc()
            )),
        )
    };
    ($class:ident, $attr:ident, $argc:literal) => {
        $class.add_attr(
            stringify!($attr),
            $crate::objref!($crate::bytecode::objects::Object::Function(
                $crate::bytecode::objects::CompiledFunction::new(
                    $argc,
                    $crate::bytecode::objects::FunctionType::Rust($attr)
                )
            )),
        )
    };
}

pub fn iter_() -> ObjectRef {
    objref!(Object::Function(CompiledFunction::new(
        1,
        FunctionType::Rust(iter)
    )))
}
pub fn iter(vm: &mut VM) -> Result<(), RuntimeError> {
    let object = vm.pop_tos();
    let object_class = object.borrow().class(vm.classes());

    // `iter()` actually works on the `__iter__` class attribute, not anything instance-related
    let iter = object_class.attr("__iter__").map_err(|_| {
        RuntimeError::new(&format!("'{}' object is not iterable", object_class.name()))
    })?;
    vm.push_tos(object);
    vm.push_tos(iter);
    vm.handle_callable_object("__iter__", 1)
}

pub fn next_() -> ObjectRef {
    objref!(Object::Function(CompiledFunction::new(
        1,
        FunctionType::Rust(next)
    )))
}
pub fn next(vm: &mut VM) -> Result<(), RuntimeError> {
    let object = vm.pop_tos();
    let object_class = object.borrow().class(vm.classes());

    // `next()` actually works on the `__next__` class attribute, not anything instance-related
    let next = object_class.attr("__next__").map_err(|_| {
        RuntimeError::new(&format!(
            "'{}' object is not an iterator",
            object_class.name()
        ))
    })?;
    vm.push_tos(object);
    vm.push_tos(next);
    vm.handle_callable_object("__next__", 1)
}

pub fn print_() -> ObjectRef {
    objref!(Object::Function(CompiledFunction::new(
        1,
        FunctionType::Rust(print)
    )))
}
pub fn print(vm: &mut VM) -> Result<(), RuntimeError> {
    let object = vm.pop_tos();
    let object_class = object.borrow().class(vm.classes());

    let str = object_class.attr("__str__");
    let output = if let Ok(str) = str {
        vm.push_tos(object);
        vm.push_tos(str);
        vm.handle_callable_object("__str__", 1)?;
        if let Object::String(ref output) = *vm.pop_tos().borrow() {
            output.clone()
        } else {
            return Err(RuntimeError::new("__str__ returned non-string"));
        }
    } else {
        format!(
            "<{} object at {:p}>",
            object_class.name(),
            &*object.borrow()
        )
    };
    println!("{output}");

    Ok(())
}

pub fn bool_() -> ObjectRef {
    objref!(Object::Function(CompiledFunction::new(
        1,
        FunctionType::Rust(bool)
    )))
}
pub fn bool(vm: &mut VM) -> Result<(), RuntimeError> {
    let object = vm.pop_tos();
    let object_class = object.borrow().class(vm.classes());

    if let Ok(bool) = object_class.attr("__bool__") {
        vm.push_tos(object);
        vm.push_tos(bool);
        vm.handle_callable_object("__bool__", 1)?;
    } else {
        vm.push_tos(objref!(Object::Boolean(true)));
    }

    Ok(())
}

pub fn len_() -> ObjectRef {
    objref!(Object::Function(CompiledFunction::new(
        1,
        FunctionType::Rust(len)
    )))
}
pub fn len(vm: &mut VM) -> Result<(), RuntimeError> {
    let object = vm.pop_tos();
    let object_class = object.borrow().class(vm.classes());

    if let Ok(len) = object_class.attr("__len__") {
        vm.push_tos(object);
        vm.push_tos(len);
        vm.handle_callable_object("__len__", 1)?;
    } else {
        return Err(RuntimeError::new(&format!(
            "'{}' object has no len()",
            object_class.name()
        )));
    }

    Ok(())
}
