use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct Row {
    string: String,
    // 全角文字にも対応した行の文字数
    len: usize,
}
// 文字列スライスからRowへの変換
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {
    // 行から指定した範囲[start..end]のみ(文字数単位)を返す
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.len());
        let start = cmp::min(start, end);

        let mut result = String::new();
        // start番目からend番目まで(書記素単位)の文字列を返す
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "/t" {
                // タブを半角空白に変換
                result.push(' ');
            } else {
                // 書記素一文字ずつを文字列スライス型として追加していく
                result.push_str(grapheme);
            }
        }
        result
    }
    pub fn len(&self) -> usize {
        self.len
    }
    // 全角文字にも対応した行の文字数を返す
    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }
    // 全角文字にも対応した、画面に収まる文字列を返す
    pub fn clip_string(&self, offset: usize, terminal_width: usize) -> String {
        let mut current_width = 0;
        let mut end_idx = 0;
        // let mut char_idx = 0;
        // 画面左側に映らない文字を削除
        let string = self.string[..]
            .graphemes(true)
            .skip(offset)
            .collect::<String>();

        for c in string.chars() {
            // 次の一文字の幅を取得
            let char_width = UnicodeWidthChar::width(c).unwrap_or(1);
            // 画面右端に到達したら
            if current_width <= terminal_width && terminal_width <= current_width + char_width {
                break;
            }
            current_width += char_width;
            end_idx += 1;
        }
        // 画面左端より左で行の文字列が終わっていた場合
        if current_width <= offset {
            return String::new();
        }
        string[..].graphemes(true).take(end_idx).collect::<String>()
    }
    // 指定した範囲[start..end] (全角文字単位)の文字列を半角文字単位で何個分かを返す
    pub fn char2pos(&self, start: usize, end: usize) -> usize {
        let string = self.render(start, end);
        UnicodeWidthStr::width(&string[..])
    }
}
