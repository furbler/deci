use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::editor::SearchDirection;
use crate::highlighting;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    // 全角文字にも対応した行の文字数
    len_full_width: usize,
}
// 文字列スライスからRowへの変換
impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len_full_width: slice.graphemes(true).count(),
        }
    }
}

impl Row {
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
            highlighting: Vec::new(),
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
    // 自身のafter文字目以降で引数の文字列が見つかったら全角文字単位での位置を返す
    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        // 指定位置が行末の時は検索結果無し
        if at > self.len_full_width || query.is_empty() {
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
    pub fn highlight(&mut self, word: Option<&str>) {
        let mut highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut matches = Vec::new();
        let mut search_index = 0;
        // 文字列の指定がある場合
        if let Some(word) = word {
            // 指定位置から検索文字列が見つかる限り繰り返す
            while let Some(search_match) = self.find(word, search_index, SearchDirection::Forward) {
                // 検索文字列が見つかった場所を保存
                matches.push(search_match);
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                {
                    // 次の検索開始位置
                    search_index = next_index;
                } else {
                    break;
                }
            }
        }

        let mut index = 0;
        // 1文字ずつハイライトを行う

        #[allow(clippy::integer_arithmetic)]
        while let Some(c) = chars.get(index) {
            // 文字列の指定がある場合
            if let Some(word) = word {
                // 指定文字列の時
                if matches.contains(&index) {
                    // 指定文字列の範囲はハイライト
                    for _ in word[..].graphemes(true) {
                        index += 1;
                        highlighting.push(highlighting::Type::Match);
                    }
                    continue;
                }
            }
            if c.is_ascii_digit() {
                highlighting.push(highlighting::Type::Number);
            } else {
                highlighting.push(highlighting::Type::None);
            }
            // 次の文字へ
            index += 1;
        }

        self.highlighting = highlighting;
    }
    // 全角文字にも対応した、画面に収まる文字列を返す
    pub fn trim_string(&self, full_width_offset: usize, half_width_area: usize) -> String {
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
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        for (index, grapheme) in string[..].graphemes(true).enumerate().take(end_idx) {
            if let Some(c) = grapheme.chars().next() {
                // 1文字の色を取得
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlighting::Type::None);
                // 前の文字と色が違う場合
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    // 色情報を付与
                    let start_highlight = if highlighting_type == &highlighting::Type::None {
                        // 属性無しの場合はデフォルトの色に戻す
                        format!("{}", termion::color::Fg(color::Reset))
                    } else {
                        format!("{}", termion::color::Fg(highlighting_type.to_color()))
                    };
                    result.push_str(&start_highlight[..]);
                }
                if c == '\t' {
                    // タブは半角空白に変換
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
        }
        // 最後に色情報をリセット
        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }
    // 指定した範囲[start..end] (全角文字単位)の文字列を半角文字単位で何個分かを返す
    pub fn full2half_width(&self, full_width_start: usize, full_width_end: usize) -> usize {
        let end = cmp::min(full_width_end, self.len());
        let start = cmp::min(full_width_start, full_width_end);
        #[allow(clippy::integer_arithmetic)]
        let string = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect::<String>();
        UnicodeWidthStr::width(&string[..])
    }
    // 指定した範囲[..end] (半角文字単位)の文字列を全角文字単位で何個分かを返す
    pub fn half2full_width(&self, half_width_end: usize) -> usize {
        let string = self.trim_string(0, half_width_end);
        UnicodeWidthStr::width(&string[..])
    }
}
