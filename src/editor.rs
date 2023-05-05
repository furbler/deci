use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

// 文字色
const STATUS_FG_COLOR: color::Rgb = color::Rgb(13, 13, 13);
// 背景色
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
// コンパイル時にバージョン情報を取得
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}
impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    // 開いているドキュメントの先頭に対する画面左上の位置
    offset: Position,
    document: Document,
    status_message: StatusMessage,
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
            // キー入力が無い間は入力待ち状態で処理は進まない
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }
    pub fn default() -> Self {
        // コマンドの引数を取得
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit");
        // 引数でファイル名が指定されていたら
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(file_name);
            // 指定されたファイル名が開ければその内容を保存
            if let Ok(doc) = doc {
                doc
            } else {
                // 失敗したらエラーメッセージを出してから、ファイル名を指定しなかったときと同じ動作をする
                initial_status = format!("ERR: Could not open file: {file_name}");
                Document::default()
            }
        } else {
            // 中身を空とする
            Document::default()
        };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
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
            self.draw_status_bar();
            self.draw_message_bar();
            // カーソルの画面上の位置を求めて、カーソルを表示する
            let char_pos = if let Some(row) = self.document.row(self.cursor_position.y) {
                row.char2pos(self.offset.x, self.cursor_position.x)
            } else {
                0
            };
            Terminal::cursor_position(&Position {
                x: char_pos,
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
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
        self.scroll();
        Ok(())
    }
    fn scroll(&mut self) {
        // キー入力による移動後のカーソル位置を取得
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        // カーソルが画面より上
        if y < offset.y {
            // カーソルを画面の一番上に置く
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            // カーソルが画面より下の時はカーソルを画面の一番下に置く
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        // カーソルが画面より左
        if x < offset.x {
            // カーソルを画面の一番左に置く
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            // カーソルが画面より右の時はカーソルを画面の一番右に置く
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    // 入力したキーに応じてカーソル移動
    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let document_height = self.document.len();
        let width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up | Key::Char('k') => y = y.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                if y < document_height {
                    y = y.saturating_add(1);
                };
            }
            Key::Left | Key::Char('h') => {
                if x > 0 {
                    // 行頭でなければ左に移動
                    x -= 1;
                } else if y > 0 {
                    // 行頭で、かつドキュメントの最初の行でない場合
                    // 1つ上の行に移動
                    y -= 1;
                    // 行末に移動
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right | Key::Char('l') => {
                if x < width {
                    x += 1;
                } else if y < document_height {
                    // 行末で、かつドキュメントの最後の行でない場合
                    // 下の行の行頭に移動
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp | Key::Ctrl('b') => {
                // 1画面分上に移動
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown | Key::Ctrl('f') => {
                // 1画面分下に移動
                y = if y.saturating_add(terminal_height) < document_height {
                    y + terminal_height
                } else {
                    document_height
                }
            }
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
        let half_width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + half_width;
        // 表示する内容を指定した範囲で切り取る
        // 文字列が画面より左側で終わっていたら空文字列が入る
        let row = row.render(start, end);
        // カーソルのある行を描画して改行する
        println!("{row}\r");
    }
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height - 1 {
            Terminal::clear_current_line();
            // 表示すべきファイルの行があれば表示する
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
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
    fn draw_status_bar(&self) {
        let mut status;
        // ファイル名が指定されなかった場合のデフォルトの表示名
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            // ファイル名で20文字を超えていた分は表示しない
            file_name.truncate(60);
        }
        // ファイル名
        status = format!("{file_name}  ");
        // カーソルのある行/総行数 (最初を1とする)
        let line_indicator = format!(
            "line: {}/{}  ",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let char_width = if let Some(row) = self.document.row(self.cursor_position.y) {
            row.len()
        } else {
            0
        };
        // カーソルの行頭からの文字数/総文字数
        let column_indicator = format!(
            "col: {}/{char_width}",
            self.cursor_position.x.saturating_add(1),
        );
        // 左端のファイル名と右端の行数表示の間は半角空白で埋める
        let len = status.len() + line_indicator.len() + column_indicator.len();
        let terminal_width = self.terminal.size().width as usize;
        if terminal_width > len {
            status.push_str(&" ".repeat(terminal_width - len));
        }
        status = format!("{status}{line_indicator}{column_indicator}");
        // 画面に収まりきらない部分は削る
        status.truncate(terminal_width);
        // 背景色、文字色を設定
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        // ステータスバー上の文字を表示
        println!("{status}\r");
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }
    fn draw_message_bar(&self) {
        // メッセージバーをクリア
        Terminal::clear_current_line();
        let message = &self.status_message;
        // メッセージが表示開始から一定時間経過するまで表示
        if message.time.elapsed() < Duration::new(5, 0) {
            let mut text = message.text.clone();
            // 画面からはみ出すメッセージ部分は削除
            text.truncate(self.terminal.size().width as usize);
            print!("{text}");
        }
    }
}

fn die(e: &std::io::Error) {
    // エラーで終了前に画面をクリア
    Terminal::clear_screen();
    panic!("{}", e);
}
