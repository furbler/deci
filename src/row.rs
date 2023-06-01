use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::editor::SearchDirection;
use crate::highlighting;
use crate::HighlightingOptions;

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
    fn highlight_match(&mut self, word: Option<&str>) {
        // 検索文字列が指定されていた場合のみハイライト追加
        if let Some(word) = word {
            // 検索文字列が空文字列の場合はハイライトなし
            if word.is_empty() {
                return;
            }
            let mut index = 0;
            // 見つかった検索文字列の位置
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
                // 見つかった検索文字列の末尾の位置
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                {
                    // 見つかった検索文字列をハイライト
                    #[allow(clippy::indexing_slicing)]
                    for i in search_match..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    // 次の検索開始地点を更新
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }
    // 指定された文字列があればハイライト
    fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        hl_type: highlighting::Type,
    ) -> bool {
        // 指定された文字列が空
        if substring.is_empty() {
            return false;
        }
        // 文字列から1文字ずつ取り出す
        for (substring_index, c) in substring.chars().enumerate() {
            // 行の指定位置から取り出して比較
            if let Some(next_char) = chars.get(index.saturating_add(substring_index)) {
                // 指定された文字列と一致しない場合はハイライトしない
                if *next_char != c {
                    return false;
                }
            } else {
                // 行末に到達した
                return false;
            }
        }
        // 指定文字列が見つかった場合
        for _ in 0..substring.len() {
            // 対応したハイライトを追加
            self.highlighting.push(hl_type);
            *index = index.saturating_add(1);
        }
        // ハイライト済み
        true
    }
    fn highlight_keywords(
        &mut self,
        index: &mut usize,
        chars: &[char],
        keywords: &[String],
        hl_type: highlighting::Type,
    ) -> bool {
        // 前の文字を取得
        if *index > 0 {
            #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
            let prev_char = chars[*index - 1];
            // 前の文字がセパレータでなかったら
            if !is_separator(prev_char) {
                // ハイライトすべきキーワードとはみなさない
                return false;
            }
        }
        // ハイライトする単語を取得
        for word in keywords {
            if *index < chars.len().saturating_sub(word.len()) {
                #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                let next_char = chars[*index + word.len()];
                // 現在位置にキーワードがあると仮定して、キーワードの後にセパレータが無い場合
                if !is_separator(next_char) {
                    // ハイライトすべきキーワードは無いと判断する
                    continue;
                }
            }
            // ハイライトした場合はtrueを返す
            if self.highlight_str(index, word, chars, hl_type) {
                return true;
            }
        }
        // ハイライトしなかった
        false
    }
    fn highlight_primary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.primary_keywords(),
            highlighting::Type::PrimaryKeywords,
        )
    }
    fn highlight_secondary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.secondary_keywords(),
            highlighting::Type::SecondaryKeywords,
        )
    }
    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        // シングルクオートに挟まれた文字にハイライトを付ける場合
        if opts.characters() && c == '\'' {
            // 次の1文字を取得
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    // 次の文字がバックスラッシュの場合は2文字間に挟んだ先の文字を取得
                    index.saturating_add(3)
                } else {
                    // 1文字間に挟んだ先の文字を取得
                    index.saturating_add(2)
                };
                // 閉じ記号を期待する位置の文字を取得
                if let Some(closing_char) = chars.get(closing_index) {
                    // 閉じ記号があったら
                    if *closing_char == '\'' {
                        // シングルクオートとそれに挟まれた文字をハイライト
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index = index.saturating_add(1);
                        }
                        // ハイライトした
                        return true;
                    }
                }
            }
        }
        // ハイライトしなかった
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        // スラッシュが見つかった場合
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                // 連続して/が存在する場合はコメントと判定
                if *next_char == '/' {
                    // 行末まで全てコメント
                    for _ in *index..chars.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *index = index.saturating_add(1);
                    }
                    // ハイライトした
                    return true;
                }
            };
        }
        // ハイライトしなかった
        false
    }
    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            // 閉じ記号が見つかるか行末に着くまで繰り返す
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index = index.saturating_add(1);
                if let Some(next_char) = chars.get(*index) {
                    // 閉じ記号が見つかったら終了
                    if *next_char == '"' {
                        break;
                    }
                } else {
                    // 行末だったら終了
                    break;
                }
            }
            self.highlighting.push(highlighting::Type::String);
            *index = index.saturating_add(1);
            // 文字列が存在した
            return true;
        }
        // 文字列が存在しない
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                #[allow(clippy::indexing_slicing, clippy::integer_arithmetic)]
                let prev_char = chars[*index - 1];
                // 一個前の文字がセパレータ
                if !is_separator(prev_char) {
                    // 数字のハイライトはしない
                    return false;
                }
            }
            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index = index.saturating_add(1);
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        // 数字またはカンマ以外が見つかったらハイライト終了
                        break;
                    }
                } else {
                    // 行末だったら終了
                    break;
                }
            }
            // 数字だった
            return true;
        }
        // 数字でなかった
        false
    }
    pub fn highlight(&mut self, opts: &HighlightingOptions, word: Option<&str>) {
        // ハイライトを初期化
        self.highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut index = 0;
        // １文字ずつ処理
        while let Some(c) = chars.get(index) {
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_primary_keywords(&mut index, opts, &chars)
                || self.highlight_secondary_keywords(&mut index, opts, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                // オーバーフローしていたら終了
                if index.checked_add(1).is_none() {
                    break;
                }
                continue;
            }
            // どのハイライトにも該当しなかった場合
            self.highlighting.push(highlighting::Type::None);
            index = index.saturating_add(1);
        }
        // 検索結果のハイライトのみ、他のハイライトを上書きする
        self.highlight_match(word);
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

fn is_separator(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_ascii_whitespace()
}
