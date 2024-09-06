#![feature(let_chains)]
#![feature(yeet_expr)]
#![feature(decl_macro)]
#![feature(try_blocks)]
#![feature(if_let_guard)]

pub mod asm;
pub mod opcode;

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
