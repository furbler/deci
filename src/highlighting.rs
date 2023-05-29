use termion::color;
#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
}
impl Type {
    // 返り値の型はimpl traitで指定
    pub fn to_color(&self) -> impl color::Color {
        match self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::Match => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(211, 54, 130),
            Type::Character => color::Rgb(108, 113, 196),
            Type::None => color::Rgb(255, 255, 255),
        }
    }
}
