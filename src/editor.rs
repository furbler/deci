use crate::Terminal;
use termion::event::Key;
// コンパイル時にバージョン情報を取得
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
}

impl Editor {
    pub fn run(&mut self) {
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
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
        }
    }
    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        // 一番左上にカーソルを置く
        Terminal::cursor_position(0, 0);
        // 終了時に画面をクリアして、メッセージを出力
        if self.should_quit {
            Terminal::clear_screen();
            println!("エディタを終了します。さようなら。\r");
        } else {
            self.draw_rows();
            // チルダ描画後にカーソルを左上に戻す
            Terminal::cursor_position(0, 0);
        }
        Terminal::cursor_show();
        // バッファの内容を出力
        Terminal::flush()
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            _ => (),
        }
        Ok(())
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

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for row in 0..height - 1 {
            Terminal::clear_current_line();
            if row == height / 3 {
                // メッセージが画面幅を超えていたら切り取る
                self.draw_welcome_message();
                // let width =
                //     std::cmp::min(self.terminal.size().width as usize, welcome_message.len());
                // println!("{}\r", &welcome_message[..width])
            } else {
                // 行頭にチルダを表示
                println!("~\r");
            }
        }
    }
}

fn die(e: std::io::Error) {
    // エラーで終了前に画面をクリア
    Terminal::clear_screen();
    panic!("{}", e);
}
