use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::VM;
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("Set");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __len__, 1);
    class_method!(class, __iter__, 1);
    class_method!(class, __contains__, 2);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Set(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(!slf.is_empty())));

    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Set(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let mut display = String::new();
    for (i, v) in slf.iter().enumerate() {
        // Try to call the value's __str__() method as well
        let v_class = v.borrow().class(vm.classes()).name();
        let v_display = if let Ok(v_str) = v.borrow().attr("__str__", vm.classes()) {
            vm.push_tos(v.clone());
            vm.push_tos(v_str);
            vm.handle_callable_object("__str__", 1)?;
            let v_display_ = vm.pop_tos();
            if let Object::String(ref v_display) = *v_display_.borrow() {
                v_display.clone()
            } else {
                return Err(RuntimeError::new("__str__ returned non-string"));
            }
        } else {
            format!("<{v_class} object at {:p}>", &*v.borrow())
        };
        if matches!(*v.borrow(), Object::String(_)) {
            display.push_str(&format!("'{v_display}'"));
        } else {
            display.push_str(&v_display);
        }

        // Only add a comma separation if there are more key-value pairs to output
        if i < slf.len() - 1 {
            display.push_str(", ");
        }
    }
    vm.push_tos(objref!(Object::String(format!("{{{display}}}"))));

    Ok(())
}

fn __len__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Set(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Number(slf.len() as f64)));
    Ok(())
}

fn __iter__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Set(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let elem_list = objref!(Object::List(slf.clone()));
    let list_iter = elem_list.borrow().attr("__iter__", vm.classes())?;
    vm.push_tos(elem_list);
    vm.push_tos(list_iter);
    vm.handle_callable_object("__iter__", 1)?;

    Ok(())
}

fn __contains__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Set(ref slf) = *slf_.borrow_mut() else {
        panic!();
    };

    let val = vm.pop_tos();
    let Ok(val_eq) = val.borrow().attr("__eq__", vm.classes()) else {
        vm.push_tos(objref!(Object::Boolean(false)));
        return Ok(());
    };

    for item in slf {
        vm.push_tos(item.clone());
        vm.push_tos(val.clone());
        vm.push_tos(val_eq.clone());
        if vm.handle_callable_object("__eq__", 2).is_ok() {
            let eq_res_ = vm.pop_tos();
            let Object::Boolean(eq_res) = *eq_res_.borrow() else {
                continue;
            };

            if eq_res {
                vm.push_tos(eq_res_);
                return Ok(());
            }
        }
    }
    vm.push_tos(objref!(Object::Boolean(false)));

    Ok(())
}
