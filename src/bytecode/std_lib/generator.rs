use crate::{
    bytecode::{
        VM,
        objects::{Class, Object},
        vm::RuntimeError,
    },
    class_method, objref,
};

pub fn init_class() -> Class {
    let mut class = Class::new("Generator");

    class_method!(class, __bool__, 1);
    class_method!(class, __iter__, 1);
    class_method!(class, __next__, 1);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Generator(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(slf.is_done())));

    Ok(())
}

fn __iter__(_vm: &mut VM) -> Result<(), RuntimeError> {
    Ok(())
}

fn __next__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Generator(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(slf.last_value());
    vm.push_tos(slf_.clone());
    vm.handle_generator()?;

    Ok(())
}
