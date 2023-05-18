use crate::Position;
use crate::Row;
use std::fs;
use std::io::Error;
use std::io::Write;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    // ローカルのファイルに対し更新があればtrue、無ければfalse
    dirty: bool,
}

impl Document {
    // 指定したファイル内容の取得に失敗したらエラーを返す
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        // 指定したファイルの中身を読み込む
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        // 一行ずつ保存する
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
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
            let new_row = self.rows[at.y].split(at.x);
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
        } else {
            // ドキュメント末尾に入力された文字を含んだ新しい行を追加
            let mut row = Row::default();
            row.insert(0, c);
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
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }
    // 上書き保存
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            // 一行ずつ保存
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            // 更新フラグを下ろす
            self.dirty = false;
        }
        Ok(())
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    // 引数の文字列を検索し、見つかった時は全角文字単位の位置を返す
    pub fn find(&self, query: &str) -> Option<Position> {
        // 一行ずつ検索
        for (y, row) in self.rows.iter().enumerate() {
            if let Some(x) = row.find(query) {
                return Some(Position { x, y });
            }
        }
        None
    }
}
