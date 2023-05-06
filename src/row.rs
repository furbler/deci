use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
        // 画面左端からの文字列
        let left_clipped_string = self.string[..]
            .graphemes(true)
            .skip(offset)
            .collect::<String>();
        // 文字列が画面に収まればそのまま表示
        if UnicodeWidthStr::width(&left_clipped_string[..]) < terminal_width {
            left_clipped_string
        } else {
            // 画面に収まらなければ、文字列がすべて2バイト文字であると仮定して切り取る
            left_clipped_string[..]
                .graphemes(true)
                .take(terminal_width / 2)
                .collect::<String>()
        }
    }
    // 指定した文字の箇所[start..end]の半角文字単位の位置を返す
    pub fn char2pos(&self, start: usize, end: usize) -> usize {
        let string = self.render(start, end);
        UnicodeWidthStr::width(&string[..])
    }
}
