use std::{fmt::Display};
use colored::Colorize;

pub enum Log {
    INF,
    WRN,
    ERR
}

macro_rules! log_inf {
    () => {
        print!("{}\n", crate::log::Log::INF)
    };
    ($($arg:tt)*) => {{
        print!("{} ", crate::log::Log::INF);
        println!($($arg)*);
    }};
}

macro_rules! label_inf {
    ($label:expr) => {
        print!("{} {}\n", crate::log::Log::INF, colored::Colorize::blue(&format!("[{}]", $label) as &str))
    };
    ($label:expr, $($arg:tt)*) => {{
        print!("{} {} ", crate::log::Log::INF, colored::Colorize::blue(&format!("[{}]", $label) as &str));
        println!($($arg)*);
    }};
}

macro_rules! log_wrn {
    () => {
        print!("{}\n", crate::log::Log::WRN)
    };
    ($($arg:tt)*) => {{
        print!("{} ", crate::log::Log::WRN);
        println!($($arg)*);
    }};
}

macro_rules! label_wrn {
    ($label:expr) => {
        print!("{} {}\n", crate::log::Log::WRN, colored::Colorize::blue(&format!("[{}]", $label) as &str))
    };
    ($label:expr, $($arg:tt)*) => {{
        print!("{} {} ", crate::log::Log::WRN, colored::Colorize::blue(&format!("[{}]", $label) as &str));
        println!($($arg)*);
    }};
}

macro_rules! log_err {
    () => {
        eprint!("{}\n", crate::log::Log::ERR)
    };
    ($($arg:tt)*) => {{
        eprint!("{} ", crate::log::Log::ERR);
        eprintln!($($arg)*);
    }};
}

macro_rules! label_err {
    ($label:expr) => {
        eprint!("{} {}\n", crate::log::Log::ERR, colored::Colorize::blue(&format!("[{}]", $label) as &str))
    };
    ($label:expr, $($arg:tt)*) => {{
        eprint!("{} {} ", crate::log::Log::ERR, colored::Colorize::blue(&format!("[{}]", $label) as &str));
        eprintln!($($arg)*);
    }};
}

pub(crate) use {log_inf, log_wrn, log_err, label_inf, label_wrn, label_err};

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = chrono::offset::Local::now();
        let now = now.format("%H:%M:%S");
        match self {
            Log::INF => write!(f, "{} {}", "[inf]".bold().cyan(), now),
            Log::WRN => write!(f, "{} {}", "[wrn]".bold().yellow(), now),
            Log::ERR => write!(f, "{} {}", "[err]".bold().red(), now),
        }
    }
}

pub trait LogExpect<T> {
    #[track_caller]
    fn log_expect (self, message: &str) -> T;
    #[track_caller]
    fn label_expect (self, label: &str, message: &str) -> T;
}

impl<T, E: std::fmt::Debug> LogExpect<T> for Result<T, E> {
    fn log_expect (self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                log_err!("{}", message);
                eprintln!("{err:?}");
                panic!();
            },
        }
    }
    fn label_expect (self, label: &str, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                label_err!(label, "{}", message);
                eprintln!("{err:?}");
                panic!();
            },
        }
    }
}

impl<T> LogExpect<T> for Option<T> {
    fn log_expect (self, message: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                log_err!("{}", message);
                panic!();
            },
        }
    }
    fn label_expect (self, label: &str, message: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                label_err!(label, "{}", message);
                panic!();
            },
        }
    }
}
