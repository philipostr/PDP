use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::VM;
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("Dict");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __len__, 1);
    class_method!(class, __getitem__, 2);
    class_method!(class, __setitem__, 3);
    class_method!(class, __delitem__, 2);
    class_method!(class, __contains__, 2);
    class_method!(class, __iter__, 1);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(!slf.is_empty())));
    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let mut display = String::new();
    for (i, (k, v)) in slf.iter().enumerate() {
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
            display.push_str(&format!("'{k}': '{v_display}'"));
        } else {
            display.push_str(&format!("'{k}': {v_display}"));
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
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Number(slf.len() as f64)));
    Ok(())
}

fn __getitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let key_ = vm.pop_tos();
    let Object::String(ref key) = *key_.borrow() else {
        return Err(RuntimeError::new("dict keys must be strings"));
    };
    if let Some((_, item)) = slf.iter().find(|(k, _)| k == key) {
        vm.push_tos(item.clone());
    } else {
        return Err(RuntimeError::new(&format!("key '{key}' not found in dict")));
    }

    Ok(())
}

fn __setitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref mut slf) = *slf_.borrow_mut() else {
        panic!();
    };

    let key_ = vm.pop_tos();
    let Object::String(ref key) = *key_.borrow() else {
        return Err(RuntimeError::new("dict keys must be strings"));
    };
    if let Some(idx) = slf.iter_mut().position(|(k, _)| k == key) {
        let new_val = vm.pop_tos();
        slf[idx].1 = new_val;
    } else {
        return Err(RuntimeError::new(&format!("key '{key}' not found in dict")));
    }

    Ok(())
}

fn __delitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref mut slf) = *slf_.borrow_mut() else {
        panic!();
    };

    let key_ = vm.pop_tos();
    let Object::String(ref key) = *key_.borrow() else {
        return Err(RuntimeError::new("dict keys must be strings"));
    };
    if let Some(idx) = slf.iter().position(|(k, _)| k == key) {
        slf.remove(idx);
    } else {
        return Err(RuntimeError::new(&format!("key '{key}' not found in dict")));
    }

    Ok(())
}

fn __contains__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let key_ = vm.pop_tos();
    let Object::String(ref key) = *key_.borrow() else {
        return Err(RuntimeError::new("dict keys must be strings"));
    };
    vm.push_tos(objref!(Object::Boolean(
        slf.iter().find(|(k, _)| k == key).is_some()
    )));

    Ok(())
}

fn __iter__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::Dict(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let key_list = objref!(Object::List(
        slf.iter()
            .map(|(k, _)| objref!(Object::String(k.clone())))
            .collect::<Vec<_>>()
    ));
    let list_iter = key_list.borrow().attr("__iter__", vm.classes())?;
    vm.push_tos(key_list);
    vm.push_tos(list_iter);
    vm.handle_callable_object("__iter__", 1)?;

    Ok(())
}
