/* ------ macro for debugging-------- */
#[macro_export]
macro_rules! DEBUG {
    ()=>(
        {
            let (file, line) = (file!(), line!());
            println!("{:30}", format!("[{}:{}]", file, line));
        }
    );
    ($($arg:tt)*) => ({
        let (file, line) = (file!(), line!());
        print!("{:30}", format!("[{}:{}]", file, line));
        println!("{:40}", format_args!($($arg)*));
    });
}
