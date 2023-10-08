use crate::info::lua::{
    ErrCode, FINE, INVOKE_RET_MISMATCH, INVOKE_STACK_OVERFLOW, LUA_MIN_STACK, LUA_MUL_RET,
    MEMORY_TYPE_MISMATCH, MEMORY_UNREACHABLE,
};
use crate::obj::objdef::{ObjectTrait, FFUNC, T_LRF};
use crate::obj::statedef::LuaState;
use crate::DEBUG;
use core::ptr::null_mut;

static mut MAINTHREAD: *mut LuaState = null_mut();

fn start() -> Result<&'static mut LuaState, ErrCode> {
    let state = LuaState::mainthread_new(null_mut());
    DEBUG!("the state is {:#?}", state.ok().unwrap());
    unsafe { MAINTHREAD = state? };
    unsafe { Ok(&mut *MAINTHREAD) }
}

pub fn get_mainthread() -> Result<&'static mut LuaState, ErrCode> {
    if unsafe { MAINTHREAD == null_mut() } {
        start()
    } else {
        unsafe { Ok(&mut *MAINTHREAD) }
    }
}

impl LuaState {
    pub fn call(&mut self, narg: usize, sresults: isize) {
        let func_index = self.get_stack_top() - (narg + 1);
        DEBUG!("func_index is: {}", func_index);
        self.call_unprotected(func_index, sresults);
        //let last = self.pop_errcode();
    }

    fn call_unprotected(&mut self, func_index: usize, sresults: isize) {
        let res = self.run(func_index, sresults);
    }

    fn run(&mut self, func_index: usize, sresults: isize) -> Result<ErrCode, ErrCode> {
        if !self.calls_check() {
            todo!()
        }
        self.pre_call(func_index, sresults)?;

        Ok(ErrCode(FINE))
    }

    fn pre_call(&mut self, func_index: usize, sresults: isize) -> Result<ErrCode, ErrCode> {
        DEBUG!("func_index: {}", func_index);
        DEBUG!("sresults: {}", sresults);
        // acquire the stack
        let stack = self.get_stack_mut_ref()?;
        // acquire the object at the index func_index
        let obj = stack.get_ref_elem(func_index)?;
        if !obj.val_idx.is_function() {
            DEBUG!("error: is not function");
            return Err(ErrCode(MEMORY_TYPE_MISMATCH));
        }
        DEBUG!("{}", obj.val_idx.into_inner());
        match obj.val_idx.into_inner() {
            T_LRF => {
                let f = Option::<FFUNC>::into_inner(obj);
                DEBUG!("{}", f.is_some());
                if let Some(function) = f {
                    self.stack_check(LUA_MIN_STACK as usize)?;
                    let frame_index = self.push_frame(func_index)?;
                    DEBUG!("frame_index: {}", frame_index);
                    let rresults = function(self);
                    DEBUG!("rresults:{}", rresults);

                    // Pop the function object from the stack
                    let _function = self.pop_stack()?;

                    // check if the top edge exceeds the boundary
                    if !self.cframe_check_stkedge(frame_index, rresults as usize)? {
                        self.write_frame_status(frame_index, ErrCode(INVOKE_STACK_OVERFLOW))?;
                        self.set_status(ErrCode(INVOKE_STACK_OVERFLOW));
                        return Err(ErrCode(INVOKE_STACK_OVERFLOW));
                    }

                    // deal with the generated frame
                    self.post_call(func_index, rresults, sresults)?;
                    DEBUG!("self.ncalls: {}", self.ncalls);
                    let _old_frame = self.pop_frame()?;
                    DEBUG!("self.ncalls: {}", self.ncalls);
                    return Ok(ErrCode(FINE));
                } else {
                    return Err(ErrCode(MEMORY_UNREACHABLE));
                }
            }
            _ => {
                return Err(ErrCode(MEMORY_TYPE_MISMATCH));
            }
        }
    }

    fn post_call(
        &mut self,
        func_index: usize,
        rresults: usize,
        sresults: isize,
    ) -> Result<ErrCode, ErrCode> {
        const ZERO_RETURN: isize = 0;
        const ONE_RETURN: isize = 1;

        match sresults {
            ZERO_RETURN => {
                self.move_top_to(func_index);
                // not really necessary, just in case
            }
            ONE_RETURN => {
                self.move_top_to(func_index + 1);
            }
            LUA_MUL_RET => {
                todo!()
            } //
            _ => return Err(ErrCode(INVOKE_RET_MISMATCH)),
        } // inside match, deal with stack
        Ok(ErrCode(FINE))
    }
}
