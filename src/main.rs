use kmd_parser::tokenizer::tokenize;

fn main() {
    let markdown_input = r#"
*Italic**Bold*Italic**
"#;

    let tokens = tokenize(markdown_input.to_string());
    println!("{:#?}", tokens);
}
