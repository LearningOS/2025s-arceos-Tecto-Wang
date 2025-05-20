//! Standard library macros

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!($($arg)*));
    }
}

/// Prints to the standard output, with a newline and color (green by default).
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\x1b[32m\n\x1b[0m") };  // Default green color
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("\x1b[32m{}\x1b[0m\n", format_args!($($arg)*)));
    }
}
