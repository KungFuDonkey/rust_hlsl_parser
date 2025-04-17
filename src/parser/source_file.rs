use std::{fs, path::{absolute, Path, PathBuf}};
use crate::parser::ParserSettings;

// SourceText
// Contains all necessary data of a text file
// Source can contain preprocessor definitions and include directives, which are handled later during tokenization 
#[derive(Default)]
pub struct SourceText
{
    pub text: Vec<u8>,
    char_index: usize
}

impl From<&str> for SourceText
{
    fn from(text: &str) -> SourceText
    {
        return Self{
            text: text.to_string().into_bytes(),
            char_index: 0
        }
    }
}

impl From<String> for SourceText
{
    fn from(text: String) -> SourceText
    {
        return Self{
            text: text.into_bytes(),
            char_index: 0
        }
    }
}

impl SourceText
{
    pub fn next_char(&mut self)
    {
        self.char_index += 1;
    }

    pub fn prev_char(&mut self)
    {
        self.char_index -= 1;
    }

    pub fn peek(&self) -> char
    {
        if self.char_index + 1 >= self.text.len()
        {
            return '\0';
        }
        return self.text[self.char_index + 1] as char;
    }

    pub fn peek_at(&self, index: usize) -> char
    {
        if self.char_index + index >= self.text.len()
        {
            return '\0';
        }
        return self.text[self.char_index + index] as char;
    }

    pub fn current_char(&self) -> char
    {
        if self.char_index >= self.text.len()
        {
            return '\0';
        }
        return self.text[self.char_index] as char;
    }

    pub fn reached_eof(&self) -> bool
    {
        return self.char_index >= self.text.len();
    }

    pub fn get_char_index(&self) -> usize
    {
        return self.char_index;
    }
}

impl ToString for SourceText
{
    fn to_string(&self) -> String 
    { 
        return String::from_utf8(self.text.clone()).expect("String was not valid utf8");
    }
}



#[derive(Default)]
pub struct SourcePath
{
    full_file_path: PathBuf, 
    file_exists: bool,
    found_paths: Vec<PathBuf>
}


impl SourcePath
{
    fn get_path(&self) -> &Path
    {
        return self.full_file_path.as_path();
    }

    fn found_multiple_paths(&self) -> bool
    {
        return self.found_paths.len() > 1;
    }

    fn from_path(path: &Path, settings: &ParserSettings) -> Self
    {
        if path.is_relative()
        {
            // For relative paths we need to find the path from the include paths
            let include_paths = &settings.include_paths;
            let mut found_paths: Vec<PathBuf> = Vec::new();
            for include_path in include_paths
            {
                let full_path = include_path.join(path);
                
                // Only say that a path was 'found' when it is a file, folders do not count as source files
                if full_path.is_file()
                {
                    match absolute(full_path.as_path())
                    {
                        Ok(abs_path) => {
                            // Check to see if the path already was found, multiple include paths could link the same files
                            if !found_paths.contains(&abs_path)
                            {
                                found_paths.push(abs_path)
                            }
                        }
                        Err(_) => {},
                    }
                }
            }

            if found_paths.len() == 0
            {
                // We did not find the file they asked for, so return that the file does not exist
                return Self::default();
            }

            // We take the first found path always
            return Self
            {
                full_file_path: found_paths[0].clone(),
                file_exists: true,
                found_paths: found_paths
            };
        }

        // Check if the file exists
        if path.is_file()
        {
            return Self
            {
                full_file_path: PathBuf::from(path),
                file_exists: true,
                found_paths: vec![PathBuf::from(path)]
            };
        }
        
        // Didn't find the file
        return Self::default();
    }
}

// ISourceFile
// Contains all filesystem information about a file
// Bindings, includes and type information is stored in ParsedFile
pub trait ISourceFile 
{
    fn get_text(&self) -> &Box<SourceText>;

    fn get_text_mut(&mut self) -> &mut Box<SourceText>;

    fn get_file_path(&self) -> &Path;
}

pub struct SourceFile
{
    text: Box<SourceText>,
    source_path: SourcePath
}

impl ISourceFile for SourceFile
{
    fn get_text(&self) -> &Box<SourceText>
    {
        return &self.text;
    }

    fn get_text_mut(&mut self) -> &mut Box<SourceText>
    {
        return &mut self.text;
    }

    fn get_file_path(&self) -> &Path
    {
        return self.source_path.get_path();
    }
}

impl SourceFile
{
    // From Text is used for testing
    pub fn from_text(text: &str) -> Self
    {
        return Self
        {
            text: Box::new(SourceText::from(text)),
            source_path: SourcePath::default()
        }
    }

    // Get it from a path
    pub fn from_path(path: &Path, settings: &ParserSettings) -> Self
    {
        let source_path = SourcePath::from_path(path, settings);
        
        if !source_path.file_exists
        {
            return Self{
                text: Box::new(SourceText::from("")),
                source_path: source_path
            }
        }

        let text = match fs::read_to_string(&source_path.full_file_path)
        {
            Ok(content) => content,
            Err(_) => String::new(),
        };

        return Self
        {
            text: Box::new(SourceText::from(text)),
            source_path: source_path
        }
    }
}
