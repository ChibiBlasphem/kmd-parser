use crate::reader::Reader;

use super::super::{helpers::{collect_until, next_char_is}, Token, TokenizerState};

pub fn get_heading(reader: &mut Reader, state: &mut TokenizerState) {
    state.current_heading_size = match state.current_heading_size {
        None => Some(1),
        Some(i) => Some(i + 1),
    };

    if next_char_is(&reader, ' ') {
        reader.next();
        let text = collect_until(reader, |ch| *ch == '\n');
        if text.trim().is_empty() {
            state.current_line = format!("{}", "#".repeat(state.current_heading_size.unwrap()));
        } else {
            state.tokens.push(Token::Heading(state.current_heading_size.unwrap(), text.trim().to_string()));
        }

        state.current_heading_size = None;
    }
}

pub fn match_heading(reader: &Reader, state: &TokenizerState) -> bool {
    let next_char = reader.peek();
    state.current_line.is_empty() && (next_char == Some('#') || next_char == Some(' '))
}