use crate::Position;
use std::io::{self, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

// 行頭の行番号の最大表示桁数 4桁+半角スペース1桁
const LINE_NUMBER_SPACES: usize = 5;

// 端末の縦横の半角文字単位のサイズ
pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    // 端末の縦横の半角文字単位のサイズ
    // 幅は端末の画面幅から行番号の表示スペースを除いたサイズ
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        #[allow(clippy::cast_possible_truncation)]
        Ok(Self {
            size: Size {
                width: size.0 - LINE_NUMBER_SPACES as u16,
                // 2行分空ける
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }
    // サイズ情報を共有参照で返す
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }
    // usizeからu16への型変換に対する警告を表示しない
    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        // カーソル位置の原点を(0, 0)で扱えるよう変換する
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        let x = x as u16;
        let y = y as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }
    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }
    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }
    // カーソルのある行のみクリアする
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }
    // 背景色を指定した色に設定
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }
    // 背景色をデフォルトの色に設定
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }
}
