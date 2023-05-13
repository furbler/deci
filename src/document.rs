use crate::Position;
use crate::Row;
use std::fs;
use std::io::Error;
use std::io::Write;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
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
        if at.y > self.len() {
            return;
        }
        // 指定位置がドキュメントの最後行の次の場合
        if at.y == self.len() {
            self.rows.push(Row::default());
        } else {
            // atで行を分割(atは後半の行に含まれる)
            let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
            // 後半行を挿入
            self.rows.insert(at.y + 1, new_row);
        }
    }
    // 指定した位置の後ろに1文字挿入
    pub fn insert(&mut self, at: &Position, c: char) {
        // Enterキーが押された時
        if c == '\n' {
            // 指定位置の下に空行を挿入
            self.insert_newline(at);
            return;
        }
        if at.y < self.len() {
            // 指定された位置の後ろに文字を挿入
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        } else {
            // ドキュメント末尾に入力された文字を含んだ新しい行を追加
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        }
    }
    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        // 指定位置がドキュメントからはみ出している時
        if at.y >= len {
            // 何もしない
            return;
        }
        // 指定位置が行の末尾にあり、かつ次の行が存在した時
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            // 指定位置の次の行を削除
            let next_row = self.rows.remove(at.y + 1);
            // 指定位置の行
            let row = self.rows.get_mut(at.y).unwrap();
            // 結合
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }
    // 上書き保存
    pub fn save(&self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            // 一行ずつ保存
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}
