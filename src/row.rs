use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

pub struct Row {
    string: String,
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
    // 行から指定した範囲のみを返す
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
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
    // 複数バイト文字にも対応した文字数を返す
    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }
}
