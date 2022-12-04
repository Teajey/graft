use crate::cross_println;

pub struct Context {
    pub config: super::config::Config,
    pub verbose: u8,
    pub config_location: Option<String>,
}

impl Context {
    pub fn print_info(&self, level: u8, msg: &str) {
        if self.verbose >= level {
            cross_println!("{msg}");
        }
    }

    // pub fn print_error(&self, msg: &str) {
    //     cross_eprintln!("{msg}");
    // }
}

#[macro_export]
macro_rules! print_info {
    ($context:ident, $level:expr, $($t:tt)*) => {
        $context.print_info($level, &format_args!($($t)*).to_string())
    }
}

#[macro_export]
macro_rules! print_error {
    ($context:ident, $($t:tt)*) => {
      context.print_error(&format_args!($($t)*).to_string())
    }
}
