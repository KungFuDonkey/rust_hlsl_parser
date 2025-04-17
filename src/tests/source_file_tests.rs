

#[cfg(test)]
mod source_file_tests {
    use std::path::PathBuf;
    use crate::parser::{source_file::{ISourceFile, SourceFile}, ParserSettings};
    use crate::tests::test_utils::create_full_path;

    // Tests if two texts are equal to each other
    fn test_same_text(source_file: &SourceFile, expected: &str)
    {
        let source_text = &source_file.get_text().to_string();
        let expected_text = &expected.to_string();

        assert_eq!(source_text, expected_text);
    }

    #[test]
    fn from_text()
    {
        let text = "void main() {}";
        let source_file = SourceFile::from_text(text);

        test_same_text(&source_file, &text);
    }

    #[test]
    fn from_full_path()
    {
        let path = create_full_path("./test_files/Simple/main_func.hlsl");
        let settings = ParserSettings::default();

        let source_file = SourceFile::from_path(path.as_path(), &settings);

        test_same_text(&source_file, "void main() {}");
    }

    #[test]
    fn from_include_path()
    {
        let path = PathBuf::from("./main_func.hlsl");
        let include_dir = create_full_path("./test_files/Simple/");
        let mut settings = ParserSettings::default();
        settings.include_paths.push(PathBuf::from(include_dir));

        let source_file = SourceFile::from_path(path.as_path(), &settings);

        test_same_text(&source_file, "void main() {}");
    }

    #[test]
    fn from_relative_path()
    {
        let path = PathBuf::from("./../main_func.hlsl");
        let include_dir = create_full_path("./test_files/Simple/Dummy");
        let mut settings = ParserSettings::default();
        settings.include_paths.push(PathBuf::from(include_dir));

        let source_file = SourceFile::from_path(path.as_path(), &settings);

        test_same_text(&source_file, "void main() {}");
    }
}
