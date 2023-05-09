use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Default)]
pub struct Row {
    string: String,
    // 全角文字にも対応した行の文字数
    len_full_width: usize,
}
// 文字列スライスからRowへの変換
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            len_full_width: 0,
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
        self.len_full_width
    }
    // 全角文字にも対応した行の文字数を返す
    fn update_len(&mut self) {
        self.len_full_width = self.string[..].graphemes(true).count();
    }
    // 指定した位置の後ろに1文字挿入する
    pub fn insert(&mut self, at: usize, c: char) {
        // 挿入位置が文字列の最後のとき
        if at >= self.len() {
            self.string.push(c);
        } else {
            // 挿入位置より前の文字列
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            // 挿入位置より後の文字列
            let remainder: String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&remainder);
            self.string = result;
        }
        // 文字列数を更新
        self.update_len();
    }

    // 全角文字にも対応した、画面に収まる文字列を返す
    pub fn clip_string(&self, full_width_offset: usize, half_width_area: usize) -> String {
        let mut current_width = 0;
        let mut end_idx = 0;
        // 画面左側に映らない文字を削除
        let string = self.string[..]
            .graphemes(true)
            .skip(full_width_offset)
            .collect::<String>();

        // 画面左端より左で行の文字列が終わっていた場合
        if string.is_empty() {
            return String::new();
        }
        for c in string.chars() {
            // 次の一文字の幅を取得
            let char_width = UnicodeWidthChar::width(c).unwrap_or(1);
            // 画面右端に到達したら
            if current_width <= half_width_area && half_width_area <= current_width + char_width {
                break;
            }
            current_width += char_width;
            end_idx += 1;
        }
        string[..].graphemes(true).take(end_idx).collect::<String>()
    }
    // 指定した範囲[start..end] (全角文字単位)の文字列を半角文字単位で何個分かを返す
    pub fn full2half_width(&self, full_width_start: usize, full_width_end: usize) -> usize {
        let string = self.render(full_width_start, full_width_end);
        UnicodeWidthStr::width(&string[..])
    }
    // 指定した範囲[..end] (半角文字単位)の文字列を全角文字単位で何個分かを返す
    pub fn half2full_width(&self, half_width_end: usize) -> usize {
        let string = self.clip_string(0, half_width_end);
        UnicodeWidthStr::width(&string[..])
    }
}
