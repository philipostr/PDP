use super::super::objects::{Class, Object};
use super::super::vm::RuntimeError;
use crate::bytecode::objects::FrozenGenerator;
use crate::bytecode::{OpCode, VM};
use crate::{class_method, objref};

pub fn init_class() -> Class {
    let mut class = Class::new("List");

    class_method!(class, __bool__, 1);
    class_method!(class, __str__, 1);
    class_method!(class, __getitem__, 2);
    class_method!(class, __iter__, 1);
    class_method!(class, __len__, 1);
    class_method!(class, __setitem__, 3);
    class_method!(class, __delitem__, 2);
    class_method!(class, __contains__, 2);

    class
}

fn __bool__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Boolean(!slf.is_empty())));

    Ok(())
}

fn __str__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow() else {
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
    vm.push_tos(objref!(Object::String(format!("[{display}]"))));

    Ok(())
}

fn __getitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let idx_ = vm.pop_tos();
    let Object::Number(idx) = *idx_.borrow() else {
        return Err(RuntimeError::new("list indices must be integers"));
    };
    let idx = if idx.is_finite() && idx.trunc() == idx {
        if idx.is_sign_negative() {
            slf.len().wrapping_sub(idx.trunc().abs() as usize)
        } else {
            idx.trunc() as usize
        }
    } else {
        return Err(RuntimeError::new("list indices must be integers"));
    };

    vm.push_tos(
        slf.get(idx)
            .ok_or(RuntimeError::new("list index out of range"))?
            .clone(),
    );

    Ok(())
}

fn __iter__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow() else {
        panic!();
    };

    let iterator = if slf.is_empty() {
        FrozenGenerator::new(Vec::new(), Vec::new(), 0, objref!(Object::None), true)
    } else if slf.len() == 1 {
        FrozenGenerator::new(
            Vec::new(),
            vec![OpCode::LOAD_CONST(0), OpCode::RETURN_VALUE],
            0,
            slf[0].clone(),
            false,
        )
    } else {
        let initial_index = Object::Number(1.0);
        let add = initial_index.attr("__add__", vm.classes()).unwrap();
        let eq = initial_index.attr("__eq__", vm.classes()).unwrap();

        FrozenGenerator::new(
            vec![
                objref!(Object::Number(1.0)),              // constant 1, doesn't change
                objref!(Object::Number(1.0)),              // index
                slf_.clone(),                              // list
                objref!(Object::Number(slf.len() as f64)), // list len
                add,                                       // number.__add__()
                eq,                                        // number.__eq__()
            ],
            vec![
                OpCode::LOAD_LOCAL(2), // Load list for use in LOAD_ACCESS
                OpCode::LOAD_LOCAL(1),
                OpCode::DUP_TOP, // Duplicate for use in LOAD_ACCESS
                OpCode::LOAD_LOCAL(3),
                OpCode::LOAD_LOCAL(5),
                OpCode::CALL_FUNCTION(3),
                OpCode::JUMP_IF_TRUE(11), // until index == len
                OpCode::LOAD_ACCESS,
                OpCode::SWAP_TOP,
                OpCode::POP_TOP,     // Remove the list from the stack
                OpCode::YIELD_VALUE, // yield list[index]
                OpCode::LOAD_LOCAL(0),
                OpCode::LOAD_LOCAL(1),
                OpCode::LOAD_LOCAL(4),
                OpCode::CALL_FUNCTION(2),
                OpCode::STORE_LOCAL(1),   // index += 1
                OpCode::JUMP_ABSOLUTE(0), // end until
                OpCode::LOAD_CONST(0),
                OpCode::RETURN_VALUE,
            ],
            0,
            slf[0].clone(),
            false,
        )
    };

    vm.push_tos(objref!(Object::Generator(iterator)));

    Ok(())
}

fn __len__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow() else {
        panic!();
    };
    vm.push_tos(objref!(Object::Number(slf.len() as f64)));
    Ok(())
}

fn __setitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref mut slf) = *slf_.borrow_mut() else {
        panic!();
    };

    let idx_ = vm.pop_tos();
    let Object::Number(idx) = *idx_.borrow() else {
        return Err(RuntimeError::new("list indices must be integers"));
    };
    let idx = if idx.is_finite() && idx.trunc() == idx {
        if idx.is_sign_negative() {
            slf.len().wrapping_sub(idx.trunc().abs() as usize)
        } else {
            idx.trunc() as usize
        }
    } else {
        return Err(RuntimeError::new("list indices must be integers"));
    };

    if idx < slf.len() {
        let new_val = vm.pop_tos();
        slf[idx] = new_val;
    } else {
        return Err(RuntimeError::new("list index out of range"));
    }

    Ok(())
}

fn __delitem__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref mut slf) = *slf_.borrow_mut() else {
        panic!();
    };

    let idx_ = vm.pop_tos();
    let Object::Number(idx) = *idx_.borrow() else {
        return Err(RuntimeError::new("list indices must be integers"));
    };
    let idx = if idx.is_finite() && idx.trunc() == idx {
        if idx.is_sign_negative() {
            slf.len().wrapping_sub(idx.trunc().abs() as usize)
        } else {
            idx.trunc() as usize
        }
    } else {
        return Err(RuntimeError::new("list indices must be integers"));
    };

    if idx < slf.len() {
        slf.remove(idx);
    } else {
        return Err(RuntimeError::new("list index out of range"));
    }

    Ok(())
}

fn __contains__(vm: &mut VM) -> Result<(), RuntimeError> {
    let slf_ = vm.pop_tos();
    let Object::List(ref slf) = *slf_.borrow_mut() else {
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
