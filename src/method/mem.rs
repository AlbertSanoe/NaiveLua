#[macro_export]
macro_rules! vec_alloc {
    ($capacity:expr) => {{
        let vec = Vec::with_capacity($capacity);
        vec
    }};
}

#[macro_export]
macro_rules! vec_push {
    ($vec:expr,$init:expr,$times:expr) => {
        for _time in 0..$times{
            $vec.push($init);
        }
    };
}

#[macro_export]
macro_rules! vec_pop {
    ($vec:expr, $times:expr) => {
        for _time in 0..$times{
            let _= $vec.pop();
        }
    };
}