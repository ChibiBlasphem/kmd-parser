mod helpers;
mod matchers;

use regex::Regex;

use crate::reader::Reader;

use self::{helpers::{get_last_paragraph, next_char_is, skip_while}, matchers::{emphasis::{get_emphasis, match_emphasis}, heading::{get_heading, match_heading}}};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Block,
    Inline,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Text(String),
    Newline,
    Heading(usize, String),
    Paragraph(Vec<Token>),
    Emphasis(usize, Vec<Token>),
    Link(String, String),
}

impl Token {
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Heading(_, _) | Token::Paragraph(_) | Token::Newline => TokenType::Block,
            Token::Text(_) | Token::Emphasis(_, _) | Token::Link(_, _) => {
                TokenType::Inline
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TokenizerState {
    current_line: String,
    tokens: Vec<Token>,
    current_heading_size: Option<usize>,
    has_stop_with_predicate: bool,
    should_start_new_paragraph: bool,
    is_inline: bool,
}

impl TokenizerState {
    pub fn new(is_inline: bool) -> Self {
        TokenizerState {
            current_line: String::new(),
            tokens: Vec::new(),
            current_heading_size: None,
            has_stop_with_predicate: false,
            should_start_new_paragraph: true,
            is_inline,
        }
    }
}

pub fn tokenize(contents: String) -> Vec<Token> {
    let mut reader = Reader::new(contents);
    let (tokens, _) = tokenize_reader(&mut reader, false, |_, _| false);
    tokens
}

fn tokenize_reader(reader: &mut Reader, is_inline: bool, stop_predicate: fn(ch: &char, tokenizer_state: &TokenizerState) -> bool) -> (Vec<Token>, bool) {
    let mut tokenizer_state = TokenizerState::new(is_inline);

    while let Some(ch) = reader.next() {
        if (is_inline && ch == '\n') || stop_predicate(&ch, &tokenizer_state) {
            tokenizer_state.has_stop_with_predicate = stop_predicate(&ch, &tokenizer_state);
            break;
        }

        match ch {
            '#' if !tokenizer_state.is_inline && match_heading(reader, &tokenizer_state) => {
                get_heading(reader, &mut tokenizer_state);
            },
            '\n' if !tokenizer_state.is_inline => {
                let mut text_token: Option<Token> = None;
                let mut has_text_newline = false;

                if !tokenizer_state.current_line.is_empty() {
                    has_text_newline = tokenizer_state.current_line.ends_with("  ");
                    if !tokenizer_state.current_line.trim().is_empty() {
                        text_token = Some(Token::Text(tokenizer_state.current_line.clone()));
                    }
                    tokenizer_state.current_line.clear();
                }

                if text_token != None || has_text_newline {
                    let paragraph_tokens = get_last_paragraph(&mut tokenizer_state.tokens, &mut tokenizer_state.should_start_new_paragraph);
                    if let Some(text) = text_token {
                        paragraph_tokens.push(text);
                    }

                    // Add newline only if next character is not a newline
                    if has_text_newline && !next_char_is(&reader, '\n') {
                        paragraph_tokens.push(Token::Newline);
                    }
                }

                let should_add_new_paragraph = match tokenizer_state.tokens.last() {
                    Some(Token::Paragraph(_)) if !next_char_is(&reader, '\n') => false,
                    Some(token) if token.token_type() == TokenType::Block => true,
                    _ => next_char_is(&reader, '\n'),
                };

                skip_while(reader, |c| *c == '\n');

                if should_add_new_paragraph {
                    tokenizer_state.should_start_new_paragraph = true
                }
            },
            '*' if match_emphasis(reader, &mut tokenizer_state) => {
                get_emphasis(reader, &mut tokenizer_state);
            }
            _ => tokenizer_state.current_line.push(ch),
        }
    }

    if !tokenizer_state.current_line.trim().is_empty() {
        let tokens_collection = if tokenizer_state.is_inline { &mut tokenizer_state.tokens } else { get_last_paragraph(&mut tokenizer_state.tokens, &mut tokenizer_state.should_start_new_paragraph) };
        tokens_collection.push(Token::Text(tokenizer_state.current_line.clone()));
    }

    (tokenizer_state.tokens, tokenizer_state.has_stop_with_predicate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_md() {
        let md_content = "# This is a heading\nWe can insert newlines\nbut they are not taken account of.  \nBut here its ok.\n\nNew paragraph?";
        assert_eq!(
            tokenize(md_content.to_string()),
            vec![
                Token::Heading(1, "This is a heading".to_string()),
                Token::Paragraph(vec![
                    Token::Text("We can insert newlines".to_string()),
                    Token::Text("but they are not taken account of.  ".to_string()),
                    Token::Newline,
                    Token::Text("But here its ok.".to_string()),
                ]),
                Token::Paragraph(vec![Token::Text("New paragraph?".to_string()),]),
            ]
        )
    }

    #[test]
    fn test_tokenize_with_inline_tokens() {
        let md_content = "# This is a heading\nWe *can* insert newlines\nbut they are not **taken account of**  \nBut here ***its ok***.\n\nNew paragraph?";
        assert_eq!(
            tokenize(md_content.to_string()),
            vec![
                Token::Heading(1, "This is a heading".to_string()),
                Token::Paragraph(vec![
                    Token::Text("We ".to_string()),
                    Token::Emphasis(1, vec![Token::Text("can".to_string())]),
                    Token::Text(" insert newlines".to_string()),
                    Token::Text("but they are not ".to_string()),
                    Token::Emphasis(2, vec![Token::Text("taken account of".to_string())]),
                    Token::Newline,
                    Token::Text("But here ".to_string()),
                    Token::Emphasis(3, vec![Token::Text("its ok".to_string())]),
                    Token::Text(".".to_string()),
                ]),
                Token::Paragraph(vec![Token::Text("New paragraph?".to_string()),]),
            ]
        )
    }
    
    #[test]
    fn test_tokenize_with_complex_inline_tokens() {
        let md_content = "# This is some heading\n\nHere is some text with a little bit of length **for testing**.\n\n*Hello this is *some* dumb shit*\n\n*****He*ll*o*****\n\n*Is it in italics*Yes it is\n\nHey this is some text with *fake italic\n\n* *First item*\n* Second item";

        assert_eq!(
            tokenize(md_content.to_string()),
            vec![
                Token::Heading(1, "This is some heading".to_string()),
                Token::Paragraph(vec![
                    Token::Text("Here is some text with a little bit of length ".to_string()),
                    Token::Emphasis(2, vec![Token::Text("for testing".to_string())]),
                    Token::Text(".".to_string())
                ]),
                Token::Paragraph(vec![
                    Token::Emphasis(1, vec![
                        Token::Text("Hello this is ".to_string()),
                        Token::Emphasis(1, vec![
                            Token::Text("some".to_string()),
                        ]),
                        Token::Text(" dumb shit".to_string()),
                    ])
                ]),
                Token::Paragraph(vec![
                    Token::Emphasis(3, vec![
                        Token::Emphasis(1, vec![
                            Token::Emphasis(1, vec![
                                Token::Text("He".to_string()),
                            ]),
                            Token::Text("ll".to_string())
                        ]),
                        Token::Text("o".to_string()),
                    ]),
                    Token::Text("**".to_string()),
                ]),
                Token::Paragraph(vec![
                    Token::Emphasis(1, vec![
                        Token::Text("Is it in italics".to_string()),
                    ]),
                    Token::Text("Yes it is".to_string()),
                ]),
                Token::Paragraph(vec![
                    Token::Text("Hey this is some text with *fake italic".to_string())
                ]),
                Token::Paragraph(vec![
                    Token::Text("* ".to_string()),
                    Token::Emphasis(1, vec![Token::Text("First item".to_string())]),
                    Token::Text("* Second item".to_string()),
                ]),
            ]
        )
    }
}
