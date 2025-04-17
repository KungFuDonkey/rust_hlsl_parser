use std::fmt::Write;

use crate::parser::syntax_tokens::SyntaxTokenType;

use super::{source_file::{ISourceFile, SourceFile, SourceText}, syntax_tokens::{get_keyword_token_type, SyntaxToken}};

pub enum PreProcessorDefinition
{
    NonFunction {name: String, value: String},
    Function {name: String, args: String, value: String}
}

// ILexer
// Generates a tokenized version of a source file, also contains pre-parsed tokens for defines and includes
// This tokenized version can be traversed by a LexerTraverser
pub trait ILexer
{
    fn get_tokens(&self) -> &Vec<SyntaxToken>;
}

pub struct Lexer
{
    tokens: Vec<SyntaxToken>
}

impl Lexer
{
    fn read_number(source: &mut SourceText) -> SyntaxTokenType
    {
        let mut sb = String::new();
        let is_octal = source.current_char() == '0' && source.peek().is_digit(8);
        let is_hex = source.current_char() == '0' && (source.peek() == 'x' || source.peek() == 'X');
        let mut has_exponential_modifier = false;
        let mut has_dot = false;
        let mut has_float_suffix = false;
        let mut base = 10;

        if is_octal
        {
            source.next_char();
            base = 8;
        }
        else if is_hex
        {
            source.next_char();
            source.next_char();
            base = 16;
        }

        while !has_float_suffix 
        {
            match source.current_char()
            {
                '.' => {
                    if is_hex || is_octal || has_dot
                    {
                        break;
                    }

                    // Check if this dot precedes a swizzle.
                    if source.peek() == 'x' || source.peek() == 'r'
                    {
                        break;
                    }

                    sb.write_char(source.current_char()).expect("concat went wrong");
                    has_dot = true;
                    source.next_char();
                },
                'e' | 'E' => 
                {
                    sb.write_char(source.current_char()).expect("concat went wrong");
                    if !is_hex
                    {
                        has_exponential_modifier = true;
                        if source.peek() == '-' || source.peek() == '+'
                        {
                            sb.write_char(source.peek()).expect("concat went wrong");
                            source.next_char();
                        }
                    }
                    source.next_char();
                },
                'f' | 'F' => 
                {
                    if !is_hex
                    {
                        has_float_suffix = !is_hex
                    }
                    else 
                    {
                        // Cannot parse 1.0f, but we can parse 1.0
                        sb.write_char(source.current_char()).expect("concat went wrong");
                    }
                    source.next_char();
                },

                'u' | 'U' | 'l' | 'L' =>
                {
                    // Can only have 1 U, but multiple LL
                    // Matches ULL, LLU, LUL
                    let mut matched_u = matches!(source.current_char(), 'u' | 'U');
                    if matches!(source.peek(), 'l' | 'L') || !matched_u && matches!(source.peek(), 'u' | 'U')
                    {
                        matched_u = matched_u || matches!(source.peek(), 'u' | 'U');
                        source.next_char();
                        if matches!(source.peek(), 'l' | 'L') || !matched_u && matches!(source.peek(), 'u' | 'U')
                        {
                            source.next_char();
                        }
                    }
                    source.next_char();
                },

                '#' =>
                {
                    if sb != "1."
                    {
                        break;
                    }

                    if source.peek_at(1) == 'I' && source.peek_at(2) == 'N' &&
                       (source.peek_at(3) == 'D' || source.peek_at(3) == 'F')
                    {
                        let is_inf = source.peek_at(3) == 'F';
                        source.next_char();
                        source.next_char();
                        source.next_char();
                        source.next_char();

                        let value = if is_inf {f64::INFINITY} else {f64::NAN};
                        
                        return SyntaxTokenType::FloatLiteralToken { value: value }
                    }
                }

                _ => {
                    if source.current_char().is_digit(base)
                    {
                        sb.write_char(source.current_char()).expect("concat went wrong");
                        source.next_char();
                    }
                    else 
                    {
                        // done
                        break;
                    }
                }
            }
        }

        if has_dot || has_exponential_modifier || has_float_suffix
        {

            let value: f64 = match sb.parse()
            {
                Ok(x) => x,
                Err(_) => 0.0
            };
            return SyntaxTokenType::FloatLiteralToken{value: value};
        }
        else 
        {
            let value: i128 = match i128::from_str_radix(&sb, base)
            {
                Ok(x) => x,
                Err(_) => 0
            };
            return SyntaxTokenType::IntegerLiteralToken{value: value};
        }
    }

    fn read_character_literal(source: &mut SourceText) -> SyntaxTokenType
    {
        
        // Skip first quote
        source.next_char();

        let c = source.current_char();
        if c == '\'' || c == '\\' || c == '\r' || c == '\n'
        {
            // report error illegal character
        }
        else 
        {
            source.next_char();
            if source.current_char() != '\''
            {
                // report error unclosed character
            }
            else
            {
                source.next_char();
            }
        }

        return SyntaxTokenType::CharacterLiteralToken{value: c}
    }

    fn read_string(source: &mut SourceText) -> SyntaxTokenType
    {
        
        // Skip first double quote
        source.next_char();

        let mut sb = String::new();

        loop
        {
            match source.current_char()
            {
                '\0' => break, // unterminated string
                '\\' => {
                    sb.write_char(source.current_char()).expect("Failed to append char");
                    source.next_char();
                    sb.write_char(source.current_char()).expect("Failed to append char");
                    source.next_char();
                },
                '"' => {
                    source.next_char();

                    if source.current_char() != '"'
                    {
                        break;
                    }

                    sb.write_char(source.current_char()).expect("Failed to append char");
                    source.next_char();
                }
                _ => {
                    sb.write_char(source.current_char()).expect("Failed to append char");
                    source.next_char();
                }
            }
        }
        
        return SyntaxTokenType::StringLiteralToken{value: sb}
    }

    fn read_identifier_or_keyword(source: &mut SourceText) -> SyntaxTokenType
    {
        let mut text = String::new();

        while source.current_char().is_alphabetic() || 
              source.current_char().is_numeric() || 
              source.current_char() == '_' || 
              source.current_char() == '$'
        {
            text.write_char(source.current_char()).expect("Failed to append char");
            source.next_char();
        }

        return get_keyword_token_type(&text)
    }

    fn read_default(source: &mut SourceText) -> SyntaxTokenType
    {
        if source.current_char().is_alphabetic() || source.current_char() == '_'
        {
            return Lexer::read_identifier_or_keyword(source);
        }
        else if source.current_char().is_digit(10)
        {
            return Lexer::read_number(source);
        }
        return SyntaxTokenType::BadToken;
    }

    fn read_eq_op(
        source: &mut SourceText, 
        original_token: SyntaxTokenType, 
        eq_token: SyntaxTokenType
    ) -> SyntaxTokenType
    {
        if source.peek() == '='
        {
            source.next_char();
            source.next_char();
            return eq_token;
        }
        source.next_char();
        return original_token;
    }

    fn read_double_char(
        source: &mut SourceText,
        original_token: SyntaxTokenType, 
        double_token: SyntaxTokenType
    ) -> SyntaxTokenType
    {
        if source.peek() == source.current_char()
        {
            source.next_char();
            source.next_char();
            return double_token;
        }
        source.next_char();
        return original_token;
    }

    fn read_double_char_or_eq_op(
        source: &mut SourceText, 
        original_token: SyntaxTokenType, 
        double_token: SyntaxTokenType, 
        eq_token: SyntaxTokenType
    ) -> SyntaxTokenType
    {
        if source.peek() == '='
        {
            source.next_char();
            source.next_char();
            return eq_token;
        }
        else if source.peek() == source.current_char()
        {
            source.next_char();
            source.next_char();
            return double_token;
        }
        source.next_char();
        return original_token;
    }

    fn read_double_double_char_or_eq_op(
        source: &mut SourceText, 
        original_token: SyntaxTokenType, 
        double_token: SyntaxTokenType,
        eq_token: SyntaxTokenType,
        double_eq_token: SyntaxTokenType,
    ) -> SyntaxTokenType
    {
        if source.peek() == '='
        {
            source.next_char();
            source.next_char();
            return eq_token;
        }
        else if source.peek() == source.current_char()
        {
            source.next_char();
            source.next_char();
            if source.current_char() == '='
            {
                return double_eq_token;
            }
            else 
            {
                return double_token;
            }
        }
        source.next_char();
        return original_token
    }

    pub fn from_text(source: &mut Box<SourceText>) -> Self
    {
        let mut tokens: Vec<SyntaxToken> = Vec::new();
        loop
        {
            use SyntaxTokenType::*;

            let start_index = source.get_char_index();

            let token = match source.current_char()
            {
                '\0' => break, // reached eof
                ' ' => WhiteSpace,
                '~'  => TildeToken,
                '&'  => Lexer::read_double_char_or_eq_op(source, AmpersandToken, AmpersandAmpersandToken, AmpersandEqualsToken),
                '|'  => Lexer::read_double_char_or_eq_op(source, BarToken, BarBarToken, BarEqualsToken),
                '^' => Lexer::read_eq_op(source, CaretToken, CaretEqualsToken),
                '?' => SyntaxTokenType::QuestionToken,
                '(' => SyntaxTokenType::OpenParenToken,
                ')' => SyntaxTokenType::CloseParenToken,
                '[' => SyntaxTokenType::OpenBracketToken,
                ']' => SyntaxTokenType::CloseBracketToken,
                '{' => SyntaxTokenType::OpenBraceToken,
                '}' => SyntaxTokenType::CloseBraceToken,
                '.' => {
                    let mut token = SyntaxTokenType::DotToken;
                    if source.peek().is_digit(10)
                    {
                        token = Lexer::read_number(source);
                    }
                    token
                },
                '+' => Lexer::read_double_char_or_eq_op(source, PlusToken, PlusPlusToken, PlusEqualsToken),
                '-' => Lexer::read_double_char_or_eq_op(source, MinusToken, MinusMinusToken, MinusEqualsToken),
                '*' => Lexer::read_eq_op(source, AsteriskToken, AsteriskEqualsToken),
                '/' => Lexer::read_eq_op(source, SlashToken, SlashEqualsToken),
                '%' => Lexer::read_eq_op(source, PercentToken, PercentEqualsToken),
                ',' => SyntaxTokenType::CommaToken,
                ';' => SyntaxTokenType::SemiToken,
                ':' => Lexer::read_double_char(source, ColonToken, ColonColonToken),
                '=' => Lexer::read_double_char(source, EqualsToken, EqualsEqualsToken),
                '!' => Lexer::read_eq_op(source, NotToken, ExclamationEqualsToken),
                '<' => Lexer::read_double_double_char_or_eq_op(source, LessThanToken, LessThanLessThanToken, LessThanEqualsToken, LessThanLessThanEqualsToken),
                '>' => Lexer::read_double_double_char_or_eq_op(source, GreaterThanToken, GreaterThanGreaterThanToken, GreaterThanEqualsToken, GreaterThanGreaterThanEqualsToken),
                '\'' => Lexer::read_character_literal(source),
                '"' => Lexer::read_string(source),
                _ => Lexer::read_default(source)
            };

            if source.get_char_index() == start_index
            {
                // Always progress
                source.next_char();
            }

            if token != WhiteSpace
            {
                let end_index = source.get_char_index();

                let syntax_token = SyntaxToken{
                    token_type: token
                };
    
                tokens.push(syntax_token);
            }
            
        }

        tokens.push(SyntaxToken{
            token_type: SyntaxTokenType::EndOfFileToken
        });

        return Lexer{
            tokens: tokens
        }
    }

    pub fn from_file(source: &mut SourceFile) -> Self
    {
        return Self::from_text(source.get_text_mut());
    }
}

impl ILexer for Lexer
{
    fn get_tokens(&self) -> &Vec<SyntaxToken> {
        return &self.tokens;
    }
}

pub trait ILexerTraverser
{
    fn next_token(&mut self) -> &SyntaxToken;
    fn peek(&self, peek_nr: usize) -> &SyntaxToken;
}

// Handles preprocessor definitions and includes after lexing
// GetNextToken therefore can skip preprocessor definitions and ONLY give the tokens that are necessary for this parse  
pub struct LexerTraverser
{
    current_token_idx: usize,
    pre_file_definitions: Vec<PreProcessorDefinition>,
    internal_file_definitions: Vec<PreProcessorDefinition>,
    tokens: Vec<SyntaxToken>,
    out_of_bounds_token: SyntaxToken
}

impl ILexerTraverser for LexerTraverser
{

    // Fetches the next token from the stream and consumes it
    fn next_token(&mut self) -> &SyntaxToken 
    {
        if self.current_token_idx >= self.tokens.len()
        {
            return &self.out_of_bounds_token;
        }
        
        let current_token = &self.tokens[self.current_token_idx];
        self.current_token_idx += 1;
        return current_token;
    }

    // Peek forward by a given number
    fn peek(&self, offset: usize) -> &SyntaxToken 
    {
        let index = self.current_token_idx + offset;
        if index >= self.tokens.len()
        {
            return &self.out_of_bounds_token;
        }

        return &self.tokens[index];
    }

    
}