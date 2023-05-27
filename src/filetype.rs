pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

#[derive(Default, Copy, Clone)]
pub struct HighlightingOptions {
    // デフォルト値はfalse
    numbers: bool,
}
impl HighlightingOptions {
    pub fn numbers(self) -> bool {
        self.numbers
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}
impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn highlighting_options(&self) -> HighlightingOptions {
        self.hl_opts
    }
    // ファイル名からファイルタイプを判断し、設定する
    pub fn from(file_name: &str) -> Self {
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions { numbers: true },
            };
        }
        Self::default()
    }
}
