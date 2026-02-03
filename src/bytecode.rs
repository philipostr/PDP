mod bytecode_emitter;
mod objects;
mod std_lib;
mod vm;

pub use bytecode_emitter::BytecodeEmitter;
pub use vm::VM;

#[allow(non_camel_case_types, clippy::upper_case_acronyms, dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    /// Do nothing.
    NOP,
    /// Pop TOS.
    POP_TOP,
    /// Swap TOS and TOS1.
    SWAP_TOP,
    /// Duplicate TOS, push duplicate onto stack.
    DUP_TOP,
    /// Pop TOS, push its inverse.
    INV_TOP,
    /// Increment IP by /0/ instructions.
    JUMP_FORWARD(usize),
    /// If TOS is falsy, increment IP by /0/ instructions. Pop TOS.
    JUMP_IF_FALSE(usize),
    /// If TOS is truthy, increment IP by /0/ instructions. Pop TOS.
    JUMP_IF_TRUE(usize),
    /// Set IP to instruction /0/.
    JUMP_ABSOLUTE(usize),
    /// Pop TOS and call iter(TOS). Generator is on the stack.
    MAKE_GENERATOR,
    /// TOS must be a generator object. If TOS.\_\_is_done\_\_ is true, pop TOS and increment IP by /0/ instructions.
    /// Otherwise, call next(TOS) and push the next value on the stack.
    FOR_ITER(usize),
    /// Store TOS in local variable /0/. Pop TOS.
    STORE_LOCAL(usize),
    /// Store TOS in deref (cell or free) variable /0/. Pop TOS.
    STORE_DEREF(usize),
    /// Store TOS in global variable with name const string /0/. Pop TOS.
    STORE_GLOBAL(usize),
    /// Store TOS in attribute of TOS1 with name const string /0/. Pop TOS.
    STORE_ATTR(usize),
    /// Store TOS in TOS2.\[TOS1\]. Uses TOS2.\_\_setitem\_\_(). Pop TOS..TOS1.
    STORE_ACCESS,
    /// Push const value /0/ onto stack.
    LOAD_CONST(usize),
    /// Push True value onto stack.
    LOAD_TRUE,
    /// Push False value onto stack.
    LOAD_FALSE,
    /// Push value in local variable /0/ onto stack.
    LOAD_LOCAL(usize),
    /// Push value in deref (cell or free) variable /0/ onto stack.
    LOAD_DEREF(usize),
    /// Push value in global variable with name const string /0/ onto stack.
    LOAD_GLOBAL(usize),
    /// Push attribute of TOS with name const string /0/ onto stack.
    LOAD_ATTR(usize),
    /// Replace TOS with TOS1\[TOS\]. Uses TOS1.\_\_getitem\_\_().
    LOAD_ACCESS,
    /// Make a function object with /0/ args and const code object /1/. Push result.
    MAKE_FUNCTION(usize, usize),
    /// Call TOS.\_\_call\_\_() with /0/ arguments. Pop TOS..TOS/0/, push result.
    CALL_FUNCTION(usize),
    /// Build a list with items TOS..TOS{ /0/-1 } in that order. Pop TOS..TOS{ /0/-1 }, push the new list.
    BUILD_LIST(usize),
    /// Build a dict with alternating keys and values TOS..TOS{ /0/-1 }. Pop TOS..TOS{ /0/-1 }, push the new dict.
    BUILD_DICT(usize),
    /// Build a set with items TOS..TOS{ /0/-1 }. Pop TOS..TOS{ /0/-1 }, push the new set.
    BUILD_SET(usize),
    /// Pop top frame, leaving the remaining (theoretically single) value from that frame on the eval stack.
    RETURN_VALUE,
    /// Pop top frame. If it was from a generator, update the generator at TOS. Otherwise, push a new generator.
    YIELD_VALUE,
    /// Push TOS onto frame's temp stack. Pop TOS.
    PUSH_TEMP,
    /// Pop top temp stack, push onto eval stack.
    POP_TEMP,
}
