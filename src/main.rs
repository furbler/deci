#![warn(clippy::all, clippy::pedantic)]
#![warn(
    clippy::indexing_slicing,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::integer_division
)]
mod document;
mod editor;
mod row;
mod terminal;
mod highlighting;

use document::Document;
use editor::Editor;
use editor::Position;
use editor::SearchDirection;
use row::Row;
use terminal::Terminal;

fn main() {
    Editor::default().run();
}
