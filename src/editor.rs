use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use termion::event::Key;

// コンパイル時にバージョン情報を取得
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            // 終了フラグが立っていたらループを抜ける
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }
    pub fn default() -> Self {
        // コマンドの引数を取得
        let args: Vec<String> = env::args().collect();
        // 引数でファイル名が指定されていたら
        let document = if args.len() > 1 {
            let file_name = &args[1];
            // 指定されたファイル名が開ければその内容を保存
            // 失敗したらファイル名を指定しなかったときと同じ動作をする
            Document::open(file_name).unwrap_or_default()
        } else {
            // 中身を空とする
            Document::default()
        };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
        }
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        // カーソルを行頭に戻す
        Terminal::cursor_position(&Position::default());
        // 終了時には画面をクリアして、メッセージを出力
        if self.should_quit {
            Terminal::clear_screen();
            println!("エディタを終了します。さようなら。\r");
        } else {
            self.draw_rows();
            Terminal::cursor_position(&self.cursor_position);
        }
        Terminal::cursor_show();
        // バッファの内容を出力
        Terminal::flush()
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            _ => self.move_cursor(pressed_key),
        }
        Ok(())
    }
    // 入力したキーに応じてカーソル移動
    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cursor_position;
        let size = self.terminal.size();
        let height = size.height.saturating_sub(1) as usize;
        let width = size.width.saturating_sub(1) as usize;
        match key {
            Key::Up | Key::Char('k') => y = y.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                if y < height {
                    y = y.saturating_add(1);
                };
            }
            Key::Left | Key::Char('h') => x = x.saturating_sub(1),
            Key::Right | Key::Char('l') => {
                if x < width {
                    x = x.saturating_add(1);
                };
            }
            Key::PageUp | Key::Ctrl('b') => y = 0,
            Key::PageDown | Key::Ctrl('f') => y = height,
            Key::Home | Key::Char('0') => x = 0,
            Key::End | Key::Char('$') => x = width,
            _ => (),
        }
        self.cursor_position = Position { x, y }
    }
    fn draw_welcome_message(&self) {
        // バージョン情報を含めたメッセージ
        let mut welcome_message = format!("Deci editor -- version {VERSION}");
        // 画面幅とメッセージ幅を計算
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        // メッセージを中央に置いたときの空けるべき余白を計算
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        // 画面中央にメッセージを表示
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        println!("{welcome_message}\r");
    }
    pub fn draw_row(&self, row: &Row) {
        let start = 0;
        let end = self.terminal.size().width as usize;
        // 表示する内容が画面からはみ出る場合は切り取る
        let row = row.render(start, end);
        // カーソルのある行を描画して改行する
        println!("{row}\r");
    }
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            Terminal::clear_current_line();
            // 表示すべきファイルの行があれば表示する
            if let Some(row) = self.document.row(terminal_row as usize) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                // ドキュメントが空であれば、1/3の高さの行にウェルカムメッセージを表示する
                self.draw_welcome_message();
            } else {
                // 行頭にチルダを表示
                println!("~\r");
            }
        }
    }
}

fn die(e: &std::io::Error) {
    // エラーで終了前に画面をクリア
    Terminal::clear_screen();
    panic!("{}", e);
}
