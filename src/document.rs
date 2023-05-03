use crate::Row;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub fn open() -> Self {
        let rows = vec![Row::from("Hello, World!")];
        Self { rows }
    }
    // 指定された行が存在すればその行をSomeで包んで、なければNoneを返す
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
}
