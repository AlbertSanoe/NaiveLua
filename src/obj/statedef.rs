use core::cell::UnsafeCell;
use core::mem::swap;
use core::ptr::null_mut;
use core::ptr::NonNull;

use crate::info::lua::MEMORY_ALLOC_FAIL;
use crate::info::lua::MEMORY_REALLOC_FAIL;
use crate::info::lua::MEMORY_TYPE_MISMATCH;
use crate::info::lua::MEMORY_UNREACHABLE;
use crate::vec_pop;
use crate::{
    info::lua::{
        ErrCode, FINE, LUA_CI_LEN, LUA_EXTRASPACE, LUA_EXTRA_STACK, LUA_MAX_CALLS, LUA_MAX_STACK,
        LUA_MIN_STACK, LUA_STACK_SIZE,
    },
    obj::objdef::{ObjectTrait, TObj, FFUNC, FLT, INT},
    ptr_get, vec_alloc, vec_push, DEBUG,
};

pub type StkElem = TObj;

#[derive(Debug)]
pub struct Stack(Vec<UnsafeCell<StkElem>>);

impl Stack {
    /// brief: alloc a new stack with capacity and length
    /// note that capacity >= length
    #[inline]
    fn new(capacity: usize, length: usize) -> Result<Stack, ErrCode> {
        // return error if length is greater than capacity
        if length > capacity {
            return Err(ErrCode(MEMORY_ALLOC_FAIL));
        }
        let mut stk = Stack {
            0: vec_alloc!(capacity),
        };
        vec_push!(stk.0, UnsafeCell::from(<StkElem>::default()), length);
        return Ok(stk);
    }

    #[inline(always)]
    pub fn get_mut_elem(&self, index: usize) -> Result<&mut StkElem, ErrCode> {
        if let Some(stk) = self.0.get(index) {
            Ok(unsafe { &mut *(stk.get()) })
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    #[inline(always)]
    pub fn get_ref_elem(&self, index: usize) -> Result<&StkElem, ErrCode> {
        if let Some(stk) = self.0.get(index) {
            Ok(unsafe { &*(stk.get()) })
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    #[inline(always)]
    pub fn get_ptr(&self, index: usize) -> Result<*mut StkElem, ErrCode> {
        if let Some(stk) = self.0.get(index) {
            Ok(stk.get())
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    #[inline(always)]
    pub fn get_elem(&self, index: usize) -> Result<StkElem, ErrCode> {
        if let Some(stk) = self.0.get(index) {
            Ok(unsafe { *(stk.get()) })
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    #[inline(always)]
    pub fn swap_elem(&self, index: usize, new_stkelem: &mut StkElem) -> Result<ErrCode, ErrCode> {
        if let Some(s) = self.0.get(index) {
            let stk = s.get();
            if !stk.is_null() {
                swap(unsafe { &mut *stk }, new_stkelem);
            } else {
                return Err(ErrCode(MEMORY_UNREACHABLE));
            }
            return Ok(ErrCode(FINE));
        } else {
            return Err(ErrCode(MEMORY_UNREACHABLE));
        }
    }

    fn increase(&mut self, need: usize) -> Result<usize, ErrCode> {
        // the space that has been allocated
        let old_alloc = self.0.len();

        if old_alloc > LUA_MAX_STACK as usize {
            return Err(ErrCode(MEMORY_REALLOC_FAIL));
        }
        // a branch that program will not step in for sure

        let mut to_add = old_alloc;
        let to_add2 = need + LUA_EXTRASPACE;

        // apply the larger one
        if to_add < to_add2 {
            to_add = to_add2;
        }

        if old_alloc + to_add > LUA_MAX_STACK as usize {
            if old_alloc + to_add2 > LUA_MAX_STACK as usize {
                return Err(ErrCode(MEMORY_REALLOC_FAIL));
            } else {
                to_add = to_add2;
            }
        }
        // capacity >= length
        vec_push!(self.0, UnsafeCell::from(<StkElem>::default()), to_add);
        return Ok(to_add);
    }

    fn decrease(&mut self, _starting_pos: usize) {}
}

#[derive(Default, Debug)]
pub struct Frame {
    stack_func_index: usize,
    stack_upper_bound: usize,
    callstatus: ErrCode,
}

impl Frame {
    fn new(stack_func_index: usize, stack_top_index: usize, status: ErrCode) -> Self {
        Self {
            stack_func_index,
            stack_upper_bound: stack_top_index,
            callstatus: status,
        }
    }

    fn fm_check_stkedge(&self, size: usize) -> bool {
        if self.stack_func_index + size >= self.stack_upper_bound {
            false
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct FrameVec(Vec<UnsafeCell<Frame>>);

impl FrameVec {
    fn new(capacity: usize, length: usize) -> Result<FrameVec, ErrCode> {
        // return error if length is greater than capacity
        if length > capacity {
            return Err(ErrCode(MEMORY_ALLOC_FAIL));
        }

        let mut frames = FrameVec {
            0: vec_alloc!(capacity),
        };

        vec_push!(frames.0, UnsafeCell::from(<Frame>::default()), length);
        Ok(frames)
    }

    fn swap_elem(&self, index: usize, new_ci: &mut Frame) -> Result<ErrCode, ErrCode> {
        if let Some(c) = self.0.get(index) {
            let ci = c.get();
            if !ci.is_null() {
                swap(unsafe { &mut *ci }, new_ci);
            } else {
                return Err(ErrCode(MEMORY_UNREACHABLE));
            }
            return Ok(ErrCode(FINE));
        } else {
            return Err(ErrCode(MEMORY_UNREACHABLE));
        }
    }

    fn increase(&mut self, civ_top_index: usize, need: usize) -> Result<usize, ErrCode> {
        // the space that has been allocated
        let old_alloc = self.0.len();

        if old_alloc > LUA_MAX_CALLS as usize {
            return Err(ErrCode(MEMORY_REALLOC_FAIL));
        }
        // will never happen

        if civ_top_index * 2 > old_alloc || civ_top_index + need > old_alloc {
            let mut to_add = old_alloc;
            let to_add2 = need;

            // apply the larger one
            if to_add < to_add2 {
                to_add = to_add2;
            }

            if old_alloc + to_add > LUA_MAX_STACK as usize {
                if old_alloc + to_add2 > LUA_MAX_STACK as usize {
                    return Err(ErrCode(MEMORY_REALLOC_FAIL));
                } else {
                    to_add = to_add2;
                }
            }
            // capacity >= length
            vec_push!(self.0, UnsafeCell::from(<Frame>::default()), to_add);
            return Ok(to_add + old_alloc);
        }
        // need add space
        else {
            return Ok(old_alloc);
        } // it is not necessary to add
    }

    fn decrease(&mut self, ncalls: usize) -> Result<usize, ErrCode> {
        let old_alloc = self.0.len();
        if old_alloc > LUA_MAX_CALLS as usize {
            return Err(ErrCode(MEMORY_REALLOC_FAIL));
        }
        if old_alloc <= LUA_CI_LEN {
            return Ok(old_alloc);
        }

        const TIMES: usize = 3;
        const DTIMES: usize = 2;
        if ncalls * TIMES <= old_alloc {
            let new_alloc = old_alloc / DTIMES;
            vec_pop!(self.0, old_alloc - new_alloc);
            return Ok(new_alloc);
        } else {
            return Ok(old_alloc);
        }
    }

    #[inline(always)]
    fn get_ref_elem(&self, index: usize) -> Result<&Frame, ErrCode> {
        if let Some(ci) = self.0.get(index) {
            Ok(unsafe { &*(ci.get()) })
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    #[inline(always)]
    fn get_mut_elem(&self, index: usize) -> Result<&mut Frame, ErrCode> {
        if let Some(ci) = self.0.get(index) {
            Ok(unsafe { &mut *(ci.get()) })
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }
}

impl Drop for FrameVec {
    fn drop(&mut self) {
        self.0.clear();
    }
}

#[derive(Default, Debug)]
struct Meta {
    pub base: UnsafeCell<Base>,
    pub global: UnsafeCell<GlobalState>,
}

static mut META: *mut Meta = null_mut();

impl Meta {
    fn new() {
        unsafe { META = Box::leak(Box::new(Meta::default())) };
        DEBUG!("META is created successfully");
    }
}

impl Drop for Meta {
    fn drop(&mut self) {
        DEBUG!("Dropping the meta..");
    }
}

#[inline(always)]
fn get_meta_mut() -> Result<*mut Meta, ErrCode> {
    if !unsafe { META.is_null() } {
        unsafe { Ok(META) }
    } else {
        Err(ErrCode(MEMORY_UNREACHABLE))
    }
}

#[macro_export]
macro_rules! get_meta {
    () => {
        Ok(unsafe { &mut *get_meta_mut()? })
    };
}

macro_rules! get_global_state {
    () => {{
        Ok((get_meta!()?).global.get_mut())
    }};
}

macro_rules! get_main_state {
    () => {{
        Ok((get_meta!()?.base.get_mut()).state.get_mut())
    }};
}

macro_rules! get_main_state_ptr {
    () => {{
        Ok((get_meta!()?.base.get_mut()).state.get())
    }};
}

#[derive(Default, Debug)]
struct Base {
    pub extra: [UnsafeCell<u8>; LUA_EXTRASPACE],
    pub state: UnsafeCell<LuaState>,
}

#[derive(Default, Debug)]
struct GlobalState {
    mainthread: Option<NonNull<LuaState>>,
    userdata: Option<NonNull<()>>,
}

#[derive(Debug, Default)]
pub struct LuaState {
    pub stack: Option<NonNull<Stack>>, // a pointer to the stack
    pub stack_last_index: usize,
    pub stack_top_index: usize, // first not used
    pub stack_size: usize,
    next: Option<NonNull<LuaState>>,     // default value: None
    previous: Option<NonNull<LuaState>>, // default value: None
    frames: Option<NonNull<FrameVec>>,
    pub ncalls: usize, // [frame]= ncalls -1
    global: Option<NonNull<GlobalState>>,
    status: ErrCode,
}

impl LuaState {
    pub fn set_status(&mut self, status: ErrCode) {
        self.status = status;
    }

    pub fn get_status(&self) -> ErrCode {
        self.status
    }

    pub fn get_stack_top(&self) -> usize {
        self.stack_top_index
    }

    pub fn change_ncalls(&mut self, step: usize, direction: bool) {
        self.ncalls = {
            if direction {
                self.ncalls + step
            } else {
                self.ncalls - step
            }
        }
    }

    pub fn write_frame_status(&self, ci_index: usize, status: ErrCode) -> Result<ErrCode, ErrCode> {
        let cci: &mut Frame = ptr_get!(self, frames)?.get_mut_elem(ci_index)?;
        cci.callstatus = status;
        return Ok(ErrCode(FINE));
    }

    pub fn get_frame_status(&self, ci_index: usize) -> Result<ErrCode, ErrCode> {
        let cci = ptr_get!(self, frames)?.get_ref_elem(ci_index)?;
        Ok(cci.callstatus)
    }

    pub fn get_stack_mut_ref(&self) -> Result<&mut Stack, ErrCode> {
        if let Some(stack) = self.stack {
            let ptr = stack.as_ptr();
            if !ptr.is_null() {
                Ok(unsafe { &mut *ptr })
            } else {
                Err(ErrCode(MEMORY_UNREACHABLE))
            }
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    pub fn get_civ_mut_ref(&self) -> Result<&mut FrameVec, ErrCode> {
        if let Some(civ) = self.frames {
            let ptr = civ.as_ptr();
            if !ptr.is_null() {
                Ok(unsafe { &mut *ptr })
            } else {
                Err(ErrCode(MEMORY_UNREACHABLE))
            }
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }

    /// initialize the stack, drop the memory manually
    fn stack_init(&mut self) -> Result<ErrCode, ErrCode> {
        // None type will return only if length size is greater that capacity
        let stk = Stack::new(LUA_MAX_STACK as usize, LUA_STACK_SIZE as usize);

        // static lifetime
        self.stack = Some(NonNull::from(Box::leak(Box::new(stk?))));

        // set the current size of the stack
        self.stack_size = LUA_STACK_SIZE as usize;
        self.stack_last_index = 0usize + (LUA_STACK_SIZE - LUA_EXTRA_STACK) as usize;

        // pos 0 is assumed to take
        self.stack_top_index = 0;

        return Ok(ErrCode(FINE));
    }

    fn stack_increase(&mut self, size: usize) -> Result<ErrCode, ErrCode> {
        let size_add = ptr_get!(self, stack)?.increase(size)?;
        self.stack_size += size_add;
        self.stack_last_index = self.stack_size - LUA_EXTRA_STACK as usize;
        Ok(ErrCode(FINE))
    }

    pub fn stack_check(&mut self, need: usize) -> Result<ErrCode, ErrCode> {
        if self.stack_top_index + need > self.stack_last_index {
            self.stack_increase(need)?;
        }
        Ok(ErrCode(FINE))
    }

    /// true: legal
    /// false: illegal
    pub fn calls_check(&self) -> bool {
        !(self.ncalls >= LUA_MAX_CALLS)
    }

    const ILLEGAL_INDEX: usize = usize::MAX;

    pub fn stack_shrink(&mut self, ci_index: usize) -> Result<ErrCode, ErrCode> {
        let cci = ptr_get!(self, frames)?.get_mut_elem(ci_index)?;
        let func_index = cci.stack_func_index;
        ptr_get!(self, stack)?.decrease(func_index);
        cci.stack_upper_bound = Self::ILLEGAL_INDEX;
        cci.stack_func_index = Self::ILLEGAL_INDEX;
        Ok(ErrCode(FINE))
    }

    fn stack_clear(&mut self) {
        drop(self.stack.take());
        self.stack = None;
        self.stack_size = 0;
        self.stack_top_index = Self::ILLEGAL_INDEX;
        self.stack_last_index = Self::ILLEGAL_INDEX;
    }

    pub fn frames_init(&mut self) -> Result<ErrCode, ErrCode> {
        // None type will return only if length size is greater that capacity
        let frames = FrameVec::new(LUA_MAX_CALLS, LUA_CI_LEN);
        // static lifetime
        let civ_box = Box::new(frames?);
        self.frames = Some(NonNull::from(Box::leak(civ_box)));
        self.ncalls = 0;
        return Ok(ErrCode(FINE));
    }

    pub fn push_frame(&mut self, func_index: usize) -> Result<usize, ErrCode> {
        // try to increase the civ
        let civ_ptr = ptr_get!(self, frames)?;

        civ_ptr.increase(self.ncalls, 1)?;

        let mut ci = Frame::new(
            func_index,
            self.stack_top_index + LUA_MIN_STACK as usize,
            ErrCode(FINE),
        );

        DEBUG!("func_index: {}", func_index);
        DEBUG!("self.stack_top_index {}", self.stack_top_index);
        DEBUG!("ci stack_top_index {}", ci.stack_upper_bound);

        civ_ptr.swap_elem(self.ncalls, &mut ci)?;
        DEBUG!("previous call val:{}", self.ncalls);
        self.ncalls += 1;
        Ok(self.ncalls - 1)
    }

    pub fn pop_frame(&mut self) -> Result<Frame, ErrCode> {
        let mut empty_frame = Frame::default();
        let frames= ptr_get!(self,frames)?;
        frames.swap_elem(self.ncalls - 1, &mut empty_frame)?;
        self.ncalls -= 1;
        let len= frames.decrease(self.ncalls)?;
        DEBUG!("new len of frames is: {}",len);
        Ok(empty_frame)
    }

    pub fn cframe_check_stkedge(&self, index: usize, size: usize) -> Result<bool, ErrCode> {
        Ok(ptr_get!(self, frames)?
            .get_ref_elem(index)?
            .fm_check_stkedge(size))
    }

    fn frames_clear(&mut self) {
        drop(self.frames.take());
        self.ncalls = 0; // no space
    }

    pub fn mainthread_new(ud: *const ()) -> Result<*mut LuaState, ErrCode> {
        // initialize meta, with default value
        Meta::new();
        // global state accepts userdata
        get_global_state!()?.userdata = Some(NonNull::from(unsafe { &*ud }));
        // link the state with global state
        get_main_state!()?.global = Some(NonNull::from(get_global_state!()?));
        // link the global state with the state
        get_global_state!()?.mainthread = Some(NonNull::from(get_main_state!()?));
        // stack initialize
        let _ = get_main_state!()?.stack_init()?;
        // civ initialize
        let _ = get_main_state!()?.frames_init()?;
        Ok(get_main_state_ptr!()?)
    }

    #[inline(always)]
    pub fn move_top_to(&mut self, index: usize) {
        self.stack_top_index = index;
    }

    #[inline(always)]
    pub fn move_top(&mut self, step: usize, direction: bool) {
        assert!(self.stack_last_index >= self.stack_top_index);
        if direction {
            self.stack_top_index += step;
        } else {
            self.stack_top_index -= step;
        }
    }

    #[inline(always)]
    fn increase_top(&mut self) {
        self.move_top(1, true);
    }

    pub fn push_errcode(&mut self, code: ErrCode) -> Result<ErrCode, ErrCode> {
        let mut elem = ErrCode::new(code);
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_integer(&mut self, integer: INT) -> Result<ErrCode, ErrCode> {
        let mut elem = Option::<INT>::new(Some(integer));
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_float(&mut self, number: FLT) -> Result<ErrCode, ErrCode> {
        let mut elem = Option::<FLT>::new(Some(number));
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_bool(&mut self, boolean: bool) -> Result<ErrCode, ErrCode> {
        let mut elem = Option::<bool>::new(Some(boolean));
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_nil(&mut self) -> Result<ErrCode, ErrCode> {
        let mut elem = Option::<()>::new(Some(()));
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_ud(&mut self, ud: Option<*mut ()>) -> Result<ErrCode, ErrCode> {
        if let Some(ud_) = ud {
            let mut elem = Option::<*mut ()>::new(Some(ud_));
            ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        } else {
            let mut elem = Option::<*mut ()>::new(Some(null_mut()));
            ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        }
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_rfunc(&mut self, rfunc: FFUNC) -> Result<ErrCode, ErrCode> {
        let mut elem = Option::<FFUNC>::new(Some(rfunc));
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn push_obj(&mut self, obj: StkElem) -> Result<ErrCode, ErrCode> {
        let mut elem = obj;
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index, &mut elem)?;
        self.increase_top();
        Ok(ErrCode(FINE))
    }

    pub fn pop_stack(&mut self) -> Result<StkElem, ErrCode> {
        let mut elem = StkElem::default();
        ptr_get!(self, stack)?.swap_elem(self.stack_top_index - 1, &mut elem)?;
        self.move_top(1, false);
        Ok(elem)
    }

    pub fn cstack_clear(&mut self, index: usize) -> Result<ErrCode, ErrCode> {
        let mut empty_elem = StkElem::default();
        ptr_get!(self, stack)?.swap_elem(index, &mut empty_elem)?;
        Ok(ErrCode(FINE))
    }

    pub fn cframe_clear(&mut self, index: usize) -> Result<ErrCode, ErrCode> {
        let mut empty_frame = Frame::default();
        ptr_get!(self, frames)?.swap_elem(index, &mut empty_frame)?;
        Ok(ErrCode(FINE))
    }

    pub fn pop_errcode(&mut self) -> Result<ErrCode, ErrCode> {
        let elem = self.pop_stack()?;
        Ok(ErrCode::into_inner(&elem))
    }

    pub fn pop_integer(&mut self) -> Result<INT, ErrCode> {
        let elem = self.pop_stack()?;
        if let Some(val) = Option::<INT>::into_inner(&elem) {
            Ok(val)
        } else {
            Err(ErrCode(MEMORY_TYPE_MISMATCH))
        }
    }

    pub fn pop_float(&mut self) -> Result<FLT, ErrCode> {
        let elem = self.pop_stack()?;
        if let Some(val) = Option::<FLT>::into_inner(&elem) {
            Ok(val)
        } else {
            Err(ErrCode(MEMORY_TYPE_MISMATCH))
        }
    }

    pub fn pop_bool(&mut self) -> Result<bool, ErrCode> {
        let elem = self.pop_stack()?;
        if let Some(val) = Option::<bool>::into_inner(&elem) {
            Ok(val)
        } else {
            Err(ErrCode(MEMORY_TYPE_MISMATCH))
        }
    }

    pub fn pop_nil(&mut self) -> Result<(), ErrCode> {
        let elem = self.pop_stack()?;
        if let Some(val) = Option::<()>::into_inner(&elem) {
            Ok(val)
        } else {
            Err(ErrCode(MEMORY_TYPE_MISMATCH))
        }
    }

    pub fn pop_ud(&mut self) -> Result<*mut (), ErrCode> {
        let elem = self.pop_stack()?;
        if let Some(val) = Option::<*mut ()>::into_inner(&elem) {
            Ok(val)
        } else {
            Err(ErrCode(MEMORY_TYPE_MISMATCH))
        }
    }
}

impl Drop for LuaState {
    fn drop(&mut self) {
        self.stack_clear();
        self.frames_clear();
    }
}
