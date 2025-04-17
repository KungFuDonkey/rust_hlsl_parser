use std::path::PathBuf;

#[derive(Default)]
pub struct ParserSettings
{
    pub include_paths: Vec<PathBuf>,
}