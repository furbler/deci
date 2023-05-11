use crate::Position;
use crate::Row;
use std::fs;

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
    // 指定した位置の後ろに1文字挿入
    pub fn insert(&mut self, at: &Position, c: char) {
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
}
