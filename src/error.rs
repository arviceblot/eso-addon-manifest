use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("missing required directive: {0}")]
    MissingDirective(String),
    #[error("invalid directive: {0}")]
    InvalidDirective(String),
    #[error("unknown directive: {0}")]
    UnknownDirective(String),
    #[error("unmapped directive: {0}")]
    UnmappedDirective(String),
    #[error("manifest file must be UTF-8 without BOM")]
    Encoding,
    #[error("directive lines beyond 301 bytes will have ignored data, line length: {0}")]
    LineLength(usize),
    #[error("comment lines are restricted to 1024 character, line length: {0}")]
    CommentLength(usize),
    #[error("Title length limited to 64 characters, current length: {0}")]
    TitleLength(usize),
    #[error("APIVersion must be at least 100003, provided: {0}")]
    ApiMinimumVersion(u32),
    #[error("error reading line")]
    ReadLineError(std::io::Error),
    #[error("unknown manifest error")]
    Unknown,
}

pub type Result<T, E = ManifestError> = std::result::Result<T, E>;
