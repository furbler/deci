use std::io::{self, stdout, Read};
use termion::raw::IntoRawMode;

fn to_ctrl_byte(c: char) -> u8 {
    let byte = c as u8;
    byte & 0b0001_1111
}

fn main() {
    let _stdout = stdout().into_raw_mode().unwrap();

    for b in io::stdin().bytes() {
        let b = b.unwrap();
        let c = b as char;
        if c.is_control() {
            // 2進数, 10進数
            println!("{:08b}, {:?}\r", b, b);
        } else {
            // 2進数, 10進数 (文字)
            println!("{:08b}, {:?} ({})\r", b, b, c);
        }

        if b == to_ctrl_byte('q') {
            break;
        }
    }
}
