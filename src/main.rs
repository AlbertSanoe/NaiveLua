use obj::statedef::LuaState;
use vm::machine::get_mainthread;

pub mod info;
pub mod method;
pub mod obj;
pub mod vm;

pub fn _main(state: &mut LuaState) -> usize {
    let k = state.pop_bool().ok().unwrap();
    let i = state.pop_integer().ok().unwrap();
    ss(k, i);
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
    DEBUG!("stack size: {}",state.stack_size);
    DEBUG!("stack_top_size: {}",state.stack_top_index);
    state.call(2, 0);
}
