#![warn(clippy::all, clippy::pedantic)]
mod editor;
mod terminal;

use editor::Editor;
use editor::Position;
use terminal::Terminal;

fn main() {
    Editor::default().run();
}
