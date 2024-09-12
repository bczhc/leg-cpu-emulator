#![feature(let_chains)]
#![feature(yeet_expr)]
#![feature(decl_macro)]
#![feature(try_blocks)]
#![feature(if_let_guard)]
#![feature(bigint_helper_methods)]

pub mod assembler;
pub mod instruction;
pub mod emulator;
pub mod components;

pub const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

trait VecExt<T>
where
    T: Copy + Sized,
{
    fn push_all(&mut self, items: impl Iterator<Item = T>);
}

impl<T> VecExt<T> for Vec<T> where T: Copy + Sized {
    fn push_all(&mut self, items: impl Iterator<Item=T>) {
        for x in items {
            self.push(x);
        }
    }
}

pub fn parse_u8_literal(s: &str) -> Option<u8> {
    if let Some(x) = s.strip_prefix("0x") {
        u8::from_str_radix(x, 16).ok()
    } else if let Some(x) = s.strip_prefix("0b") {
        u8::from_str_radix(x, 2).ok()
    } else {
        s.parse::<u8>().ok()
    }
}
