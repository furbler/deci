use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;
use unicode_segmentation::UnicodeSegmentation;

// ステータスバー文字色
const STATUS_FG_COLOR: color::Rgb = color::Rgb(13, 13, 13);
// ステータスバー背景色
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
// 行番号背景色
const LINE_NUMBER_BG_COLOR: color::Rgb = color::Rgb(53, 53, 53);
// コンパイル時にバージョン情報を取得
const VERSION: &str = env!("CARGO_PKG_VERSION");
// 行頭の行番号の最大表示桁数 4桁+半角スペース1桁
const LINE_NUMBER_SPACES: usize = 5;
// 変更を未保存のまま終了するときの終了コマンド回数
const QUIT_TIMES: u8 = 3;

#[derive(Default, Clone)]
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
    // trueはノーマルモード、falseはインサートモード
    vim_normal_mode: bool,
    terminal: Terminal,
    cursor_position: Position,
    // 開いているドキュメントの先頭に対する画面左上の位置
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
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
        // 起動直後にステータスバーに表示するメッセージ
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        // 引数でファイル名が指定されていたら
        let document = if let Some(file_name) = args.get(1) {
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
            vim_normal_mode: true,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
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
                row.full2half_width(self.offset.x, self.cursor_position.x)
            } else {
                0
            };
            Terminal::cursor_position(&Position {
                x: (char_pos).saturating_add(LINE_NUMBER_SPACES),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        // バッファの内容を出力
        Terminal::flush()
    }
    // ファイルに保存
    fn save(&mut self) {
        // エディタ起動時にファイル名が指定されてい場合
        if self.document.file_name.is_none() {
            // ファイル名入力を促す
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            // ファイル名が指定されていなければ
            if new_name.is_none() {
                // メッセージを表示
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                // 何もしない
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            // 成功
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            // 失敗
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }
    // 文字列検索
    fn search(&mut self) {
        // 検索開始前にカーソルの位置を保存
        let old_position = self.cursor_position.clone();
        // 検索文字列を取得
        if let Some(query) = self
            .prompt("Search: ", |editor, _, query| {
                // 改行またはEscが入力されるまでループ
                // 文字が入力されるたびに検索文字列の位置にカーソルをジャンプ
                if let Some(position) = editor.document.find(query) {
                    editor.cursor_position = position;
                    editor.scroll();
                }
            })
            .unwrap_or(None)
        {
            // 入力した検索文字列が見つかった場合
            if let Some(position) = self.document.find(&query[..]) {
                // カーソルを検索文字列の先頭に移動
                self.cursor_position = position;
            } else {
                // 検索文字列が見つからなかった場合
                self.status_message = StatusMessage::from(format!("Not found :{query}."));
            }
        } else {
            // 何も入力されない、またはEscでキャンセルされた場合
            // 検索開始前の位置にカーソルを戻す
            self.cursor_position = old_position;
            self.scroll();
        }
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                // 更新有りで終了しようとしたときは入力を促すメッセージを表示するのみ
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times = self.quit_times.saturating_sub(1);
                    return Ok(());
                }
                // 更新無し、またはCtrl-Qを規定回数押されたときは終了
                self.should_quit = true;
            }
            Key::Ctrl('s') => self.save(),
            // ノーマルモード時に/で検索
            Key::Char('/') if self.vim_normal_mode => self.search(),
            // Enterキーが押されたとき
            Key::Char('\n') => {
                self.document.insert(&self.cursor_position, '\n');
                // カーソルを下に移動
                self.move_cursor(Key::Down);
            }
            // 挿入モード時に任意の文字が入力されたとき
            Key::Char(c) if !self.vim_normal_mode => {
                // その文字を挿入してからカーソルを移動
                self.document.insert(&self.cursor_position, c);
                // カーソルを右に移動
                self.move_cursor(Key::Right);
            }
            // ノーマルモード時にiを入力したら挿入モードに移行
            Key::Char('i') if self.vim_normal_mode => self.vim_normal_mode = false,
            // ノーマルモードに移行
            Key::Esc => self.vim_normal_mode = true,
            // Deleteキー、またはノーマルモード時にxを押したらカーソル位置の文字を削除
            //  挿入モードでxを押した時は、上のアームでマッチするのでここはマッチしない
            Key::Delete | Key::Char('x') => {
                self.document.delete(&self.cursor_position);
            }
            Key::Backspace => {
                // カーソルがドキュメントの先頭でなければ
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    // カーソルを一つ前に移動
                    self.move_cursor(Key::Left);
                    // 挿入モードの時のみ
                    if !self.vim_normal_mode {
                        // 文字を削除
                        self.document.delete(&self.cursor_position);
                    }
                }
            }
            _ => self.move_cursor(pressed_key),
        }
        self.scroll();
        // 終了コマンドを規定回数入力前に他の入力があったらカウントをリセット
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
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
                    x = x.saturating_sub(1);
                } else if y > 0 {
                    // 行頭で、かつドキュメントの最初の行でない場合
                    // 1つ上の行に移動
                    y = y.saturating_sub(1);
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
                    x = x.saturating_add(1);
                } else if y < document_height {
                    // 行末で、かつドキュメントの最後の行でない場合
                    // 下の行の行頭に移動
                    y = y.saturating_add(1);
                    x = 0;
                }
            }
            Key::PageUp | Key::Ctrl('b') => {
                // 1画面分上に移動
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown | Key::Ctrl('f') => {
                // 1画面分下に移動
                y = if y.saturating_add(terminal_height) < document_height {
                    y.saturating_add(terminal_height)
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
    // カーソルが画面の外側に外れたら画面をスクロールさせる
    fn scroll(&mut self) {
        // キー入力による移動後のカーソル位置を取得
        let Position { x, y } = self.cursor_position;
        let terminal_width = self.terminal.size().width as usize;
        let terminal_height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        // カーソルが画面より上
        if y < offset.y {
            // カーソルを画面の一番上に置く
            offset.y = y;
        } else if y >= offset.y.saturating_add(terminal_height) {
            // カーソルが画面より下の時はカーソルを画面の一番下に置く
            offset.y = y.saturating_sub(terminal_height).saturating_add(1);
        }

        if let Some(row) = self.document.row(y) {
            // 半角単位でのカーソル位置と画面のオフセットを取得
            let half_cursor_x = row.full2half_width(0, x);
            let half_offset_x = row.full2half_width(0, offset.x);
            // カーソルが画面より左
            if x < offset.x {
                // カーソルを画面の一番左に置く
                offset.x = x;
            } else if half_offset_x.saturating_add(terminal_width) <= half_cursor_x {
                // カーソルが画面右端より右にある時はカーソルを画面の一番右に置く
                offset.x = row.half2full_width(half_cursor_x.saturating_sub(terminal_width));
            }
        }
    }
    fn draw_welcome_message(&self) {
        // バージョン情報を含めたメッセージ
        let mut welcome_message = format!("Deci editor -- version {VERSION}");
        // 画面幅とメッセージ幅を計算
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        // メッセージを中央に置いたときの空けるべき余白を計算
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        // 画面中央にメッセージを表示
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        println!("{welcome_message}\r");
    }
    pub fn draw_row(&self, row: &Row) {
        let half_width = self.terminal.size().width as usize;
        // 表示する内容を指定した範囲で切り取る
        // offsetは全角文字単位、terminal_widthは半角文字単位
        let row = row.clip_string(self.offset.x, half_width);
        // カーソルのある行を描画して改行する
        println!("{row}\r");
    }
    #[allow(clippy::integer_division, clippy::integer_arithmetic)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            let line_number = terminal_row as usize + self.offset.y;
            // 表示すべきファイルの行があれば表示する
            if let Some(row) = self.document.row(line_number) {
                // 表示する行番号が5桁以上の場合は下4桁だけ表示する
                draw_line_number((line_number + 1) % 10000);
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
        // 更新されていた場合
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
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
        #[allow(clippy::integer_arithmetic)]
        let show_len =
            status.len() + line_indicator.len() + column_indicator.len() + modified_indicator.len();
        // 行番号表示スペースも考慮する
        let terminal_width =
            (self.terminal.size().width as usize).saturating_add(LINE_NUMBER_SPACES);
        // 左端のファイル名と右端の行数表示の間は半角空白で埋める
        status.push_str(&" ".repeat(terminal_width.saturating_sub(show_len)));

        status = format!("{status}{line_indicator}{column_indicator}{modified_indicator}");
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
            text.truncate((self.terminal.size().width as usize).saturating_add(LINE_NUMBER_SPACES));
            print!("{text}");
        }
    }
    // 引数の文字列を表示してから文字入力を受け付け、入力された文字を返す
    fn prompt<C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: Fn(&mut Self, Key, &String),
    {
        let mut result = String::new();
        // 改行またはEscが入力されるまでループ
        loop {
            // プロンプト表示
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;

            // 1文字ずつ読み込む
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    // 最後の1文字を削除
                    result = result[..]
                        .graphemes(true)
                        .take(result[..].graphemes(true).count().saturating_sub(1))
                        .collect();
                }
                // 改行が入力されたら入力終了
                Key::Char('\n') => break,
                Key::Char(c) => {
                    // 入力文字が制御文字でなければ追加
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    // それまでの入力内容を破棄して終了
                    result = String::new();
                    break;
                }
                _ => (),
            }
            // 入力されるたびに実行される
            callback(self, key, &result);
        }
        // ステータスメッセージを初期化
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        // 入力された文字列を返す
        Ok(Some(result))
    }
}

// 右揃え空白詰めで行番号表示
fn draw_line_number(line_number: usize) {
    Terminal::set_bg_color(LINE_NUMBER_BG_COLOR);
    // 行番号表示の後に半角スペースを1つ入れる
    print!(
        "{line_number:>digits_width$} ",
        digits_width = LINE_NUMBER_SPACES.saturating_sub(1)
    );
    Terminal::reset_bg_color();
}

fn die(e: &std::io::Error) {
    // エラーで終了前に画面をクリア
    Terminal::clear_screen();
    panic!("{}", e);
}
