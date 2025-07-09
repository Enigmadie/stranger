#[derive(Debug, PartialEq)]
pub enum FileVariant {
    Directory,
    File,
}

#[derive(Debug)]
pub struct FileEntry {
    pub name: String,
    pub variant: FileVariant,
}
