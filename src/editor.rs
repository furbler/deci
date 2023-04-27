use std::io::{self, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Editor {}

impl Editor {
    pub fn run(&self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        for key in io::stdin().keys() {
            match key {
                Ok(key) => match key {
                    // 任意の文字
                    Key::Char(c) => {
                        let b = c as u8;
                        if c.is_control() {
                            // 2進数, 10進数
                            println!("{:08b}, {:?}\r", b, b);
                        } else {
                            // 2進数, 10進数 (文字)
                            println!("{:08b}, {:?} ({})\r", b, b, c);
                        }
                    }
                    // Ctrl-q
                    Key::Ctrl('q') => break,
                    // 上記以外
                    _ => println!("{:?}\r", key),
                },
                Err(err) => die(err),
            }
        }
    }

    pub fn default() -> Self {
        Self {}
    }
}

fn die(e: std::io::Error) {
    panic!("{}", e);
}
