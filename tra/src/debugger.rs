use std::fmt::{Arguments, Result, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        print!("{}", s);
        Ok(())
    }
}

pub fn dprint(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! error {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if cfg!(feature = "error") {
            $crate::debugger::dprint(format_args!(concat!(
                "\x1b[31m", "[error] ", $fmt, "\x1b[0m", "\n"
            ) $(, $($arg)+)?));
        }
    }
}

#[macro_export]
macro_rules! warn {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if cfg!(feature = "warn") {
            $crate::debugger::dprint(format_args!(concat!(
                "\x1b[93m", "[warn] ", $fmt, "\x1b[0m", "\n"
            ) $(, $($arg)+)?));
        }
    }
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if cfg!(feature = "info") {
            $crate::debugger::dprint(format_args!(concat!(
                "\x1b[34m", "[info] ", $fmt, "\x1b[0m", "\n"
            ) $(, $($arg)+)?));
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        if cfg!(feature = "debug") {
            $crate::debugger::dprint(format_args!(concat!(
                "\x1b[32m", "[debug] ", $fmt, "\x1b[0m", "\n"
            ) $(, $($arg)+)?));
        }
    }
}