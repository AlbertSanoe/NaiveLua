use core::mem::size_of;

type Err = u32;

#[derive(Debug, Clone, Copy)]
pub struct ErrCode(pub Err);

impl Default for ErrCode {
    fn default() -> Self {
        Self(FINE)
    }
}

// basic error
pub const FINE: Err = 0;
pub const ERR_INVOKE: Err = 1;
pub const ERR_MEMORY: Err = 2;
pub const ERR_CONCURENCY: Err = 3;
pub const ERR_COMPILE: Err = 4;
pub const ERR_OPTIMIZE: Err = 5;

pub const BASIC_ERROR_BITS: usize = 4;

// invocation error
pub const INVOKE_RET_MISMATCH: Err = 1 << BASIC_ERROR_BITS | ERR_INVOKE;
pub const INVOKE_STACK_OVERFLOW: Err = 2 << BASIC_ERROR_BITS | ERR_INVOKE;
pub const INVOKE_FRAME_OVERFLOW: Err = 3 << BASIC_ERROR_BITS | ERR_INVOKE;

// memory access error
pub const MEMORY_ALLOC_FAIL: Err = 1 << BASIC_ERROR_BITS | ERR_MEMORY;
pub const MEMORY_REALLOC_FAIL: Err = 2 << BASIC_ERROR_BITS | ERR_MEMORY;
pub const MEMORY_UNREACHABLE: Err = 3 << BASIC_ERROR_BITS | ERR_MEMORY;
pub const MEMORY_MODIFY_FAIL: Err = 6 << BASIC_ERROR_BITS | ERR_MEMORY;
pub const MEMORY_TYPE_MISMATCH: Err = 7 << BASIC_ERROR_BITS | ERR_MEMORY;
pub const MEMORY_DROP_FAIL: Err = 8 << BASIC_ERROR_BITS | ERR_MEMORY;

pub const NULL_POINTER: Err = 1;
pub const NONE_OBJECT: Err = 2;
pub const OVERFLOW: Err = 3;
// R[3-0]

pub const STATE_OK: Err = 0 << 4;
pub const STATE_ERR_ERR: Err = 1 << 4;
pub const LUA_ERR_MEM: Err = 2 << 4; // failed allocating memory
pub const STATE_ERR_RUN: Err = 3 << 4;
// R[7-4]

pub const CALL_OK: Err = 0 << 8;
pub const TOO_MANY_CALL: Err = 1 << 8;
pub const STACK_OVERFLOW: Err = 2 << 8;
pub const CALL_MISMATCHED: Err = 3 << 8;
// R[11-8]

pub const LUA_MIN_STACK: u32 = 20; // for callinfo structure
pub const LUA_STACK_SIZE: u32 = 2 * LUA_MIN_STACK; // initial stack size
pub const LUA_EXTRA_STACK: u32 = 5;
pub const LUA_MAX_STACK: u32 = 15000;
pub const LUA_ERROR_STACK: u32 = 200;

pub const LUA_MUL_RET: isize = -1;
pub const LUA_MAX_CALLS: usize = 200;
pub const LUA_CI_LEN: usize = 20; // need not pop out
pub const LUA_EXTRASPACE: usize = size_of::<*mut ()>();
