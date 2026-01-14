mod bytecode_emitter;
mod objects;
mod std_lib;

pub use bytecode_emitter::BytecodeEmitter;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum OpCode {
    /// Do nothing.
    NOP,
    /// Pop TOS.
    POP_TOP,
    /// Swap TOS and TOS1.
    SWAP_TOP,
    /// Duplicate TOS, push duplicate onto stack.
    DUP_TOP,
    /// Increment IP by /0/ instructions.
    JUMP_FORWARD(usize),
    /// If TOS is false, increment IP by /0/ instructions. Pop TOS.
    JUMP_IF_FALSE(usize),
    /// If TOS is true, increment IP by /0/ instructions. Pop TOS.
    JUMP_IF_TRUE(usize),
    /// Set IP to instruction /0/.
    JUMP_ABSOLUTE(usize),
    /// Call TOS.\_\_iter\_\_() and pop TOS. Generator is on the stack.
    ///
    /// Alias for
    /// ```
    /// LOAD_ATTR("__iter__")
    /// CALL_FUNCTION(0)
    /// POP_TOP
    /// POP_TOP
    /// ```
    /// with more specific error messages.
    MAKE_GENERATOR,
    /// If TOS.\_\_is_done\_\_ is true, pop TOS and increment IP by /0/ instructions.
    /// Otherwise, call TOS.\_\_next\_\_(). Next value is on the stack.
    ///
    /// Alias for
    /// ```
    /// LOAD_ATTR("__is_done__")
    /// JUMP_IF_FALSE(4)
    /// POP_TOP
    /// POP_TOP
    /// JUMP_FORWARD(/0/ + 2)
    /// LOAD_ATTR("__next__")
    /// CALL_FUNCTION(0)
    /// ```
    /// with more specific error messages.
    FOR_ITER(usize),
    /// Store TOS in local variable /0/. Pop TOS.
    STORE_LOCAL(usize),
    /// Store TOS in deref (cell or free) variable /0/. Pop TOS.
    STORE_DEREF(usize),
    /// Store TOS in global variable with name const string /0/. Pop TOS.
    STORE_GLOBAL(usize),
    /// Store TOS in attribute of TOS1 with name const string /0/. Pop TOS.
    STORE_ATTR(usize),
    /// Store TOS in TOS2.\[TOS1\]. Uses TOS2.\_\_index\_\_(). Pop TOS and TOS1.
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
    /// Replace TOS with TOS1\[TOS\]. Uses TOS1.\_\_index\_\_().
    LOAD_ACCESS,
    /// Make a function object with /0/ args and code object TOS. Pop TOS, push result.
    MAKE_FUNCTION(usize),
    /// Call TOS.\_\_call\_\_() with /0/ arguments. Pop TOS..TOS/0/, push result.
    CALL_FUNCTION(usize),
    /// Build a list with items TOS..TOS{ /0/-1 } in that order. Pop TOS..TOS{ /0/-1 }, push the new list.
    BUILD_LIST(usize),
    /// Build a dict with alternating keys and values TOS..TOS{ /0/-1 }. Pop TOS..TOS{ /0/-1 }, push the new dict.
    BUILD_DICT(usize),
    /// Build a set with items TOS..TOS{ /0/-1 }. Pop TOS..TOS{ /0/-1 }, push the new set.
    BUILD_SET(usize),
    /// Push TOS onto next frame's stack, pop top frame.
    RETURN_VALUE,
    /// Push TOS onto frame's temp stack. Pop TOS.
    PUSH_TEMP,
    /// Pop top of frame's temp stack, push onto stack.
    POP_TEMP,
}
