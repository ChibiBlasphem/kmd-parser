use crate::reader::Reader;

use super::Token;

pub fn collect_until(reader: &mut Reader, predicate: fn(ch: &char) -> bool) -> String {
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

pub fn next_char_is(reader: &Reader, ch: char) -> bool {
    reader.peek() == Some(ch)
}

pub fn skip_while(reader: &mut Reader, predicate: fn(ch: &char) -> bool) {
    while let Some(ch) = reader.peek() {
        if !predicate(&ch) {
            break;
        }
        reader.next();
    }
}

pub fn get_last_paragraph<'a>(tokens: &'a mut Vec<Token>, force_create: &mut bool) -> &'a mut Vec<Token> {
    let has_paragraph = if let Some(Token::Paragraph(_)) = tokens.last() {
        true
    } else {
        false
    };

    // If there's no paragraph token, add one
    if !has_paragraph || *force_create {
        *force_create = false;
        tokens.push(Token::Paragraph(vec![]));
    }

    // Return a mutable reference to the last token
    if let Some(Token::Paragraph(paragraph_tokens)) = tokens.last_mut() {
        paragraph_tokens
    } else {
        unreachable!("Failed to get the last token");
    }
}