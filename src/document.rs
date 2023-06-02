use crate::FileType;
use crate::Position;
use crate::Row;
use crate::SearchDirection;
use std::fs;
use std::io::Error;
use std::io::Write;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    // ローカルのファイルに対し更新があればtrue、無ければfalse
    dirty: bool,
    file_type: FileType,
}

impl Document {
    // 指定したファイル内容の取得に失敗したらエラーを返す
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        // 指定したファイルの中身を読み込む
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        let mut rows = Vec::new();
        // 行がコメントから始まるか否か
        let mut start_with_comment = false;
        // 一行ずつ保存する
        for value in contents.lines() {
            let mut row = Row::from(value);
            // 行全体のハイライトを行う
            start_with_comment =
                row.highlight(file_type.highlighting_options(), None, start_with_comment);
            rows.push(row);
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
            file_type,
        })
    }
    // ファイルタイプ名を返す
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }
    // 指定された行が存在すればその行をSomeで包んで、なければNoneを返す
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    // ドキュメントの総行数を返す
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    // ドキュメントに行を挿入
    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        // 指定位置がドキュメントの最後行の次の場合
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
        } else {
            // atで行を分割(atは後半の行に含まれる)
            #[allow(clippy::indexing_slicing)]
            let current_row = &mut self.rows[at.y];
            let mut new_row = current_row.split(at.x);
            // 分割前後の行をハイライト
            current_row.highlight(self.file_type.highlighting_options(), None, false);
            new_row.highlight(self.file_type.highlighting_options(), None, false);
            // 後半行を挿入
            #[allow(clippy::integer_arithmetic)]
            self.rows.insert(at.y + 1, new_row);
        }
    }
    // 指定した位置の後ろに1文字挿入
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }
        // 更新フラグを立てる
        self.dirty = true;
        // Enterキーが押された時
        if c == '\n' {
            // 指定位置の下に空行を挿入
            self.insert_newline(at);
            return;
        }
        if at.y < self.rows.len() {
            // 指定された位置の後ろに文字を挿入
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
            row.highlight(self.file_type.highlighting_options(), None, false);
        } else {
            // ドキュメント末尾に入力された文字を含んだ新しい行を追加
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(self.file_type.highlighting_options(), None, false);
            self.rows.push(row);
        }
    }
    #[allow(clippy::integer_arithmetic, clippy::indexing_slicing)]
    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        // 指定位置がドキュメントからはみ出している時
        if at.y >= len {
            // 何もしない
            return;
        }
        // 更新フラグを立てる
        self.dirty = true;
        // 指定位置が行の末尾にあり、かつ次の行が存在した時
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            // 指定位置の次の行を削除
            let next_row = self.rows.remove(at.y + 1);
            // 指定位置の行
            let row = &mut self.rows[at.y];
            // 結合
            row.append(&next_row);
            row.highlight(self.file_type.highlighting_options(), None, false);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
            row.highlight(self.file_type.highlighting_options(), None, false);
        }
    }
    pub fn save(&mut self) -> Result<(), Error> {
        // ファイル名取得
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = FileType::from(file_name);
            // 行がコメントから始まるか否か
            let mut start_with_comment = false;
            // 一行ずつ保存
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
                // ハイライト更新
                start_with_comment = row.highlight(
                    self.file_type.highlighting_options(),
                    None,
                    start_with_comment,
                );
            }
            // 更新フラグを下ろす
            self.dirty = false;
        }
        Ok(())
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    // 指定された位置から引数の文字列を検索し、見つかった時は全角文字単位の位置を返す
    // queryに空文字列を指定するとNoneを返す
    #[allow(clippy::indexing_slicing)]
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        // atがドキュメントの範囲外の時は何もしない
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = Position { x: at.x, y: at.y };
        // 検索方向により検索範囲を決める
        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };
        for _ in start..end {
            // 一行取り出す
            if let Some(row) = self.rows.get(position.y) {
                // 行内検索で見つかったらその位置を返す
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                // 見つからなかった場合
                if direction == SearchDirection::Forward {
                    // 次の行の先頭に移動
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    // 前の行の末尾に移動
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                // 検索範囲の端まで見つからなかったら終了
                return None;
            }
        }
        None
    }
    pub fn highlight(&mut self, word: Option<&str>) {
        let mut start_with_comment = false;
        for row in &mut self.rows {
            start_with_comment = row.highlight(
                self.file_type.highlighting_options(),
                word,
                start_with_comment,
            );
        }
    }
}
