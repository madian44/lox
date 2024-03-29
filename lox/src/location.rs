#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileLocation {
    pub line_number: u32,
    pub line_offset: u32,
}

impl FileLocation {
    pub fn new(line_number: u32, line_offset: u32) -> Self {
        FileLocation {
            line_number,
            line_offset,
        }
    }
}

pub trait ProvideLocation {
    fn start(&self) -> &FileLocation;
    fn end(&self) -> &FileLocation;
}
