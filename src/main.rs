use obj::statedef::LuaState;
use vm::machine::get_mainthread;

pub mod info;
pub mod method;
pub mod obj;
pub mod vm;

pub fn _main(state: &mut LuaState) -> usize {
    let k = state.get_bool_fromtop(0).ok().unwrap();
    let i = state.get_integer_fromtop(1).ok().unwrap();
    ss(k, i);
    {
        state.push_rfunc(tt1).ok().unwrap();
        state.push_integer(9).ok().unwrap();
        state.push_bool(false).ok().unwrap();
        state.call(2,0);
    }
    state.clear_frame_stk(2).ok().unwrap();
    return 0;
}

pub fn tt1(state: &mut LuaState)-> usize{
    let k = state.get_bool_fromtop(0).ok().unwrap();
    let i = state.get_integer_fromtop(1).ok().unwrap();
    ss(k, i);
    state.clear_frame_stk(2).ok().unwrap();
    return 0;
}

fn ss(i: bool, k: i32) {
    println!("the value is {},{}", k, i);
}

fn main() {
    let state= get_mainthread().ok().unwrap();
    state.push_rfunc(_main).ok().unwrap();
    state.push_integer(99999).ok().unwrap();
    state.push_bool(true).ok().unwrap();
    state.call(2, 0);
}
