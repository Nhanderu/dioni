use super::args_parsing::ARGS;
use std::io::{stdout, Write};

#[macro_export]
macro_rules! cond_print {
    ($($arg:tt)*) => ({
        if !ARGS.quiet {
            print!($($arg)*);
        }
    })
}

#[macro_export]
macro_rules! cond_println {
    () => ({
        if !ARGS.quiet {
            println!();
        }
    });
    ($($arg:tt)*) => ({
        if !ARGS.quiet {
            println!($($arg)*);
        }
    })
}

pub fn clear_stdout_line() {
    cond_print!("\r");
    let _ = stdout().flush();
}
