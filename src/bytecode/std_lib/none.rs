use crate::{
    bytecode::{
        VM,
        objects::{Class, Object},
        vm::RuntimeError,
    },
    class_method, objref,
};

pub fn init_class() -> Class {
    let mut class = Class::new("NoneType");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __eq__, 2);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    vm.pop_tos();
    vm.push_tos(objref!(Object::Boolean(false)));

    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    vm.pop_tos();
    vm.push_tos(objref!(Object::String("None".to_string())));

    Ok(())
}

fn __eq__(vm: &mut VM) -> Result<(), RuntimeError> {
    vm.pop_tos();

    let other_ = vm.pop_tos();
    let Object::None = *other_.borrow() else {
        let other_class = other_.borrow().class(vm.classes()).name();
        return Err(RuntimeError::new(&format!(
            "`'NoneType' == '{other_class}'` is not a supported operation"
        )));
    };

    vm.push_tos(objref!(Object::Boolean(true)));

    Ok(())
}
