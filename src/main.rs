#![warn(clippy::all, clippy::pedantic)]
mod document;
mod editor;
mod row;
mod terminal;

use document::Document;
use editor::Editor;
use editor::Position;
use row::Row;
use terminal::Terminal;

fn main() {
    Editor::default().run();
}
