

#[cfg(test)]
mod lexer_tests {
    use core::f64;
    use std::{io::Write, path::PathBuf};

    use crate::{parser::{lexer::{ILexer, Lexer}, source_file::{ISourceFile, SourceFile}, syntax_tokens::SyntaxTokenType, ParserSettings}, tests::test_utils::{find_all_shader_paths, find_project_dir}};

    fn lex_text(text: &str) -> Lexer
    {
        let mut source_file = SourceFile::from_text(text);
        let lex = Lexer::from_file(&mut source_file);
        assert!(source_file.get_text().reached_eof());
        return lex;
    }

    fn integer_literal_test(text: &str, value: i128)
    {
        let lex = lex_text(text);
        let tokens = lex.get_tokens();

        assert_eq!(tokens.len(), 2);
        let token = tokens[0].clone();

        assert_eq!(token.token_type, SyntaxTokenType::IntegerLiteralToken { value: value });
    }

    fn output_lex(lexer: &Lexer, path: &PathBuf)
    {
        let tokens = lexer.get_tokens();
        
        let token_string = tokens.into_iter().map(|i| i.to_string() + "\n").collect::<String>();

        let mut file = std::fs::File::create(path).expect("Failed to open file");

        file.write_all(token_string.as_bytes()).expect("Failed to writefile");
    }

    #[test]
    fn integer_literal_tests()
    {
        integer_literal_test("42", 42);     // Decimal literal
        integer_literal_test("052", 42);    // Octal literal
        integer_literal_test("0x2a", 0x2a); // Hex literal
        integer_literal_test("0X2A", 0x2A); // Hex literal
        integer_literal_test("42u", 42);
        integer_literal_test("42U", 42);
        integer_literal_test("42l", 42);
        integer_literal_test("42L", 42);
        integer_literal_test("42ul", 42);
        integer_literal_test("42UL", 42);
        integer_literal_test("42lu", 42);
    }

    fn float_literal_test(text: &str, value: f64)
    {
        let lex = lex_text(text);
        let tokens = lex.get_tokens();

        assert_eq!(tokens.len(), 2);
        let token = tokens[0].clone();


        match token.token_type
        {
            SyntaxTokenType::FloatLiteralToken{value: v} => {
                assert!(v == value || v.is_infinite() || v.is_nan())
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn float_literal_tests()
    {
        float_literal_test("1.0", 1.0);
        float_literal_test("1.0f", 1.0);   
        float_literal_test("1.#IND", f64::NAN); 
        float_literal_test("1.#INF", f64::INFINITY);
    }

    fn lex_file(shader: &PathBuf)
    {
        let settings = ParserSettings::default();
        let mut source_file = SourceFile::from_path(&shader, &settings);
        let lex = Lexer::from_file(&mut source_file);
        assert!(source_file.get_text().reached_eof());
        assert_ne!(lex.get_tokens().len(), 0);

        for token in lex.get_tokens()
        {
            assert!(token.token_type != SyntaxTokenType::BadToken, "Found bad token in {}", shader.to_str().unwrap());
        }

        let lex_file_string = String::from(shader.to_str().unwrap()) + ".lex";
        let lex_file = PathBuf::from(lex_file_string);
        output_lex(&lex, &lex_file);
    }

    #[test]
    fn can_lex_specific_shader()
    {
        let project_dir = find_project_dir();
        let shader = project_dir.join("test_files").join("Dxc").join("mesh.hlsl");
        lex_file(&shader);
    }

    #[test]
    fn can_lex_shaders()
    {
        let shaders = find_all_shader_paths();
        
        for shader in shaders
        {
            lex_file(&shader);
        }
    }
}