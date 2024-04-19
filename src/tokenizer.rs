use regex::Regex;

use crate::reader::Reader;

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
    Important(usize, Vec<Token>),
    Link(String, String),
}

impl Token {
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Heading(_, _) | Token::Paragraph(_) | Token::Newline => TokenType::Block,
            Token::Text(_) | Token::Important(_, _) | Token::Link(_, _) => {
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
}

impl TokenizerState {
    pub fn new() -> Self {
        TokenizerState {
            current_line: String::new(),
            tokens: Vec::new(),
            current_heading_size: None,
            has_stop_with_predicate: false,
            should_start_new_paragraph: true,
        }
    }
}

pub fn tokenize(contents: String) -> Vec<Token> {
    let mut reader = Reader::new(contents);
    let (tokens, _) = tokenize_reader(&mut reader, false, |_, _| false);
    tokens
}

fn tokenize_reader(reader: &mut Reader, is_inline: bool, stop_predicate: fn(ch: &char, tokenizer_state: &TokenizerState) -> bool) -> (Vec<Token>, bool) {
    let mut tokenizer_state = TokenizerState::new();

    while let Some(ch) = reader.next() {
        if (is_inline && ch == '\n') || stop_predicate(&ch, &tokenizer_state) {
            // println!("stopped {:?} {:?}", stop_predicate(&ch, &current_line), &current_line);
            tokenizer_state.has_stop_with_predicate = stop_predicate(&ch, &tokenizer_state);
            break;
        }

        match ch {
            '#' if !is_inline && tokenizer_state.current_line.is_empty()
                && (next_char_is(&reader, '#') || next_char_is(&reader, ' ')) =>
            {
                tokenizer_state.current_heading_size = match tokenizer_state.current_heading_size {
                    None => Some(1),
                    Some(i) => Some(i + 1),
                };

                if next_char_is(&reader, ' ') {
                    reader.next();
                    let text = collect_until(reader, |ch| *ch == '\n');
                    if text.trim().is_empty() {
                        tokenizer_state.current_line = format!("{}", "#".repeat(tokenizer_state.current_heading_size.unwrap()));
                    } else {
                        tokenizer_state.tokens.push(Token::Heading(tokenizer_state.current_heading_size.unwrap(), text.trim().to_string()));
                    }

                    tokenizer_state.current_heading_size = None;
                }
            },
            '*' => {
                // If next char is not a space then try to get an italic
                if !next_char_is(reader, ' ') {
                    let mut sub_reader = reader.clone();
                    let (important_tokens, status) = tokenize_reader(&mut sub_reader, true, |ch, ts| {
                        let re = Regex::new(r"\S$").unwrap();
                        *ch == '*' && (ts.tokens.len() > 0 || ts.current_line.len() > 0 && re.is_match(ts.current_line.as_str()))
                    });
                    
                    if status {
                        reader.set_index(sub_reader.index());
                        
                        let important_token = if let [Token::Important(strength, sub_tokens)] = &important_tokens[..] {
                            Token::Important(strength + 1, sub_tokens.to_vec())
                        } else {
                            Token::Important(1, important_tokens)
                        };

                        if is_inline {
                            if !tokenizer_state.current_line.trim().is_empty() {
                                tokenizer_state.tokens.push(Token::Text(tokenizer_state.current_line.trim().to_string()));
                            }
                            tokenizer_state.current_line.clear();
                            tokenizer_state.tokens.push(important_token);
                        } else {
                            let paragraph = get_last_paragraph(&mut tokenizer_state.tokens, &mut tokenizer_state.should_start_new_paragraph);
                            if let Token::Paragraph(paragraph_tokens) = paragraph {
                                if !tokenizer_state.current_line.trim().is_empty() {
                                    paragraph_tokens.push(Token::Text(tokenizer_state.current_line.trim().to_string()));
                                }
                                tokenizer_state.current_line.clear();
                                paragraph_tokens.push(important_token);
                            }
                        }
                        continue;
                    }
                }

                tokenizer_state.current_line.push(ch);
            }
            '\n' if !is_inline => {
                let mut text_token: Option<Token> = None;
                let mut has_text_newline = false;

                if !tokenizer_state.current_line.is_empty() {
                    has_text_newline = tokenizer_state.current_line.ends_with("  ");
                    if !tokenizer_state.current_line.trim().is_empty() {
                        text_token = Some(Token::Text(tokenizer_state.current_line.trim().to_string()));
                    }
                    tokenizer_state.current_line.clear();
                }

                if text_token != None || has_text_newline {
                    let paragraph = get_last_paragraph(&mut tokenizer_state.tokens, &mut tokenizer_state.should_start_new_paragraph);
                    if let Token::Paragraph(paragraph_tokens) = paragraph {
                        if let Some(text) = text_token {
                            paragraph_tokens.push(text);
                        }

                        // Add newline only if next character is not a newline
                        if has_text_newline && !next_char_is(&reader, '\n') {
                            paragraph_tokens.push(Token::Newline);
                        }
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
            _ => tokenizer_state.current_line.push(ch),
        }
    }

    if !tokenizer_state.current_line.trim().is_empty() {
        if is_inline {
            tokenizer_state.tokens.push(Token::Text(tokenizer_state.current_line.trim().to_string()));
        } else {
            let paragraph = get_last_paragraph(&mut tokenizer_state.tokens, &mut tokenizer_state.should_start_new_paragraph);
            if let Token::Paragraph(paragraph_tokens) = paragraph {
                paragraph_tokens.push(Token::Text(tokenizer_state.current_line.trim().to_string()));
            }
        }
    }

    (tokenizer_state.tokens, tokenizer_state.has_stop_with_predicate)
}

fn skip_while(reader: &mut Reader, predicate: fn(ch: &char) -> bool) {
    while let Some(ch) = reader.peek() {
        if !predicate(&ch) {
            break;
        }
        reader.next();
    }
}

fn collect_until(reader: &mut Reader, predicate: fn(ch: &char) -> bool) -> String {
    let mut text = String::new();
    while let Some(ch) = reader.peek() {
        if predicate(&ch) {
            break;
        }

        text.push(ch);
        reader.next();
    }
    text
}

fn next_char_is(reader: &Reader, ch: char) -> bool {
    reader.peek() == Some(ch)
}

fn get_last_paragraph<'a>(tokens: &'a mut Vec<Token>, force_create: &mut bool) -> &'a mut Token {
    if !*force_create {
        if let Some(Token::Paragraph(_)) = tokens.last_mut() {
            return tokens.last_mut().unwrap();
        }
    }
    
    *force_create = false;
    tokens.push(Token::Paragraph(vec![]));
    tokens.last_mut().unwrap()
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
                    Token::Text("but they are not taken account of.".to_string()),
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
                    Token::Text("We".to_string()),
                    Token::Important(1, vec![Token::Text("can".to_string())]),
                    Token::Text("insert newlines".to_string()),
                    Token::Text("but they are not".to_string()),
                    Token::Important(2, vec![Token::Text("taken account of".to_string())]),
                    Token::Newline,
                    Token::Text("But here".to_string()),
                    Token::Important(3, vec![Token::Text("its ok".to_string())]),
                    Token::Text(".".to_string()),
                ]),
                Token::Paragraph(vec![Token::Text("New paragraph?".to_string()),]),
            ]
        )
    }

    fn test_tokenize_with_complex_inline_tokens() {
        let md_content = "# This is some heading\n\nHere is some text with a little bit of length **for testing**.\n\n*Hello this is *some* dumb shit*\n\n*****He*ll*o*****\n\n*Is it in italics*Yes it is\n\nHey this is some text with *fake italic\n\n* *First item*\n* Second Item";

        assert_eq!(
            tokenize(md_content.to_string()),
            vec![
                Token::Heading(1, "This is some heading".to_string()),
                Token::Paragraph(vec![
                    Token::Text("Here is some text with a little bit of length".to_string()),
                    Token::Important(2, vec![Token::Text("for testing".to_string())]),
                    Token::Text(".".to_string())
                ]),
                Token::Paragraph(vec![
                    Token::Important(1, vec![
                        Token::Text("Hello this is".to_string()),
                        Token::Important(1, vec![
                            Token::Text("some".to_string()),
                        ]),
                        Token::Text("dumb shit".to_string()),
                    ])
                ]),
                Token::Paragraph(vec![
                    Token::Important(5, vec![
                        Token::Text("He".to_string()),
                        Token::Important(1, vec![Token::Text("ll".to_string())]),
                        Token::Text("lo".to_string()),
                    ]), 
                ]),
                Token::Paragraph(vec![
                    Token::Important(1, vec![
                        Token::Text("Is it in italics".to_string()),
                    ]),
                    Token::Text("Yes it is".to_string()),
                ]),
                Token::Paragraph(vec![
                    Token::Text("Hey this is some text with *fake italic".to_string())
                ]),
                Token::Paragraph(vec![
                    Token::Text("* ".to_string()),
                    Token::Important(1, vec![Token::Text("First item".to_string())]),
                    Token::Text("* Second item".to_string()),
                ]),
            ]
        )
    }
}
