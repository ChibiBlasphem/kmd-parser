use regex::Regex;

use crate::{reader::Reader, tokenizer::{helpers::get_last_paragraph, tokenize_reader, Token, TokenizerState}};

pub fn get_emphasis(reader: &mut Reader, state: &mut TokenizerState) {
    let mut sub_reader = reader.clone();
    let (important_tokens, status) = tokenize_reader(&mut sub_reader, true, |ch, ts| {
        let re = Regex::new(r"\S$").unwrap();
        *ch == '*' && (ts.tokens.len() > 0 || ts.current_line.len() > 0 && re.is_match(ts.current_line.as_str()))
    });
    
    if status {
        reader.set_index(sub_reader.index());
        
        let important_token = if let [Token::Emphasis(strength, sub_tokens)] = &important_tokens[..] {
            Token::Emphasis(strength + 1, sub_tokens.to_vec())
        } else {
            Token::Emphasis(1, important_tokens)
        };

        let tokens_collection = if state.is_inline { &mut state.tokens } else { get_last_paragraph(&mut state.tokens, &mut state.should_start_new_paragraph) };
        if !state.current_line.trim().is_empty() {
            tokens_collection.push(Token::Text(state.current_line.clone()));
        }
        state.current_line.clear();
        tokens_collection.push(important_token);

        return;
    }

    state.current_line.push('*');
}

pub fn match_emphasis(reader: &Reader, _state: &TokenizerState) -> bool {
    reader.peek() != Some(' ')
}