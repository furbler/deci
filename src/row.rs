use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::editor::SearchDirection;

#[derive(Default)]
pub struct Row {
    string: String,
    // 全角文字にも対応した行の文字数
    len_full_width: usize,
}
// 文字列スライスからRowへの変換
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len_full_width: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    // 行から指定した範囲[start..end]のみ(文字数単位)を返す
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.len());
        let start = cmp::min(start, end);

        let mut result = String::new();
        // start番目からend番目まで(書記素単位)の文字列を返す
        #[allow(clippy::integer_arithmetic)]
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
    // 指定した位置の後ろに1文字挿入する
    pub fn insert(&mut self, at: usize, c: char) {
        // 挿入位置が文字列の最後のとき
        if at >= self.len() {
            self.string.push(c);
            // 文字列数を更新
            self.len_full_width = self.len_full_width.saturating_add(1);
            return;
        }
        let mut result: String = String::new();
        let mut length: usize = 0;
        // 1文字ずつ処理
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length = length.saturating_add(1);
            // 指定位置では文字を挿入
            if index == at {
                length = length.saturating_add(1);
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len_full_width = length;
        self.string = result;
    }
    pub fn delete(&mut self, at: usize) {
        // カーソルが行の最後にある時
        if at >= self.len() {
            // 何もしない
            return;
        }
        let mut result: String = String::new();
        let mut length: usize = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            // 指定された位置の文字のみスキップ
            if index != at {
                length = length.saturating_add(1);
                result.push_str(grapheme);
            }
        }
        self.len_full_width = length;
        self.string = result;
    }
    // 自身の後ろに指定された行を結合する
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len_full_width = self.len_full_width.saturating_add(new.len_full_width);
    }
    // 指定位置で行を分割し、後半の行を返す
    pub fn split(&mut self, at: usize) -> Self {
        // 前半行
        let mut row: String = String::new();
        let mut length: usize = 0;
        // 後半行
        let mut splitted_row: String = String::new();
        let mut splitted_length: usize = 0;
        // 1文字ずつ処理
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            // 指定された位置の前後で割り振る
            if index < at {
                length = length.saturating_add(1);
                row.push_str(grapheme);
            } else {
                splitted_length = splitted_length.saturating_add(1);
                splitted_row.push_str(grapheme);
            }
        }
        // 前半行
        self.string = row;
        self.len_full_width = length;
        // 後半行
        Self {
            string: splitted_row,
            len_full_width: splitted_length,
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
    // 自身のafter文字目以降で引数の文字列が見つかったら全角文字単位での位置を返す
    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        // 指定位置が行末の時は検索結果無し
        if at > self.len_full_width {
            return None;
        }
        // 検索方向により検索範囲を決める
        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.len_full_width
        } else {
            at
        };
        // 指定範囲の文字列を取得
        #[allow(clippy::integer_arithmetic)]
        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };
        // 検索文字列が見つかった場合
        if let Some(matching_byte_index) = matching_byte_index {
            // 文字の半角単位の位置と全角単位の位置を比較
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }
    // 全角文字にも対応した、画面に収まる文字列を返す
    pub fn clip_string(&self, full_width_offset: usize, half_width_area: usize) -> String {
        let mut current_width = 0;
        let mut end_idx: usize = 0;
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
            if current_width <= half_width_area
                && half_width_area <= current_width.saturating_add(char_width)
            {
                break;
            }
            current_width = current_width.saturating_add(char_width);
            end_idx = end_idx.saturating_add(1);
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
