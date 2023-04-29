use std::io::{self, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) {
        let _stdout = stdout().into_raw_mode().unwrap();
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            // 終了フラグが立っていたらループを抜ける
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        Self { should_quit: false }
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        // 画面をクリアして、一番左上にカーソルを置く
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        // 終了時にメッセージを出力
        if self.should_quit {
            println!("エディタを終了します。さようなら。\r");
        }
        // バッファの内容を出力
        io::stdout().flush()
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            _ => (),
        }
        Ok(())
    }
}
fn read_key() -> Result<Key, std::io::Error> {
    loop {
        if let Some(key) = io::stdin().lock().keys().next() {
            return key;
        }
    }
}
fn die(e: std::io::Error) {
    // エラーで終了前に画面をクリア
    print!("{}", termion::clear::All);
    panic!("{}", e);
}
