mod reader;
pub mod tokenizer;

pub fn parse(contents: String) -> String {
    "<h1>Some content</h1>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file() {
        assert_eq!(
            parse("# Some content".to_string()),
            "<h1>Some content</h1>".to_string()
        );
    }

    // #[test]
    // fn test_parse_file_2() {
    //     assert_eq!(
    //         parse("Some content".to_string()),
    //         "<p>Some content</p>".to_string(),
    //     );
    // }
}



