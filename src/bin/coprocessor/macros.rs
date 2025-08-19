#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        ::std::eprintln!("{}", <::colored::ColoredString as ::colored::Colorize>::bold(<&str as ::colored::Colorize>::bright_blue(::std::format!($($arg)*).as_str())));
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        ::std::eprintln!("{}", <::colored::ColoredString as ::colored::Colorize>::bold(<&str as ::colored::Colorize>::bright_red(::std::format!($($arg)*).as_str())));
    }
}
