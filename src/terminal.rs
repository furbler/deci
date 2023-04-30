// 端末の縦横の文字数
pub struct Size {
    pub width: u16,
    pub height: u16,
}
pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1,
            },
        })
    }
    // サイズ情報を参照で返す
    pub fn size(&self) -> &Size {
        &self.size
    }
}
