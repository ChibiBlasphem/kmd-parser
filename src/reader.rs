#[derive(Debug)]
pub struct Reader {
    index: Option<usize>,
    contents: String,
}

impl Clone for Reader {
    fn clone(&self) -> Self {
        Reader {
            index: self.index,
            contents: self.contents.clone(),
        }
    }
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let index = match self.index {
            Some(i) => i + 1,
            None => 0,
        };

        self.index = Some(index);
        self.contents.chars().nth(index)
    }
}

impl Reader {
    pub fn new(contents: String) -> Self {
        Reader {
            index: None,
            contents,
        }
    }

    pub fn peek<T>(&self) -> Option<T>
    where
        T: Copy,
        Self: Iterator<Item = T>,
    {
        self.clone().peekable().peek().cloned()
    }

    pub fn current(&self) -> Option<char> {
        if let Some(i) = self.index {
            return self.contents.chars().nth(i);
        }
        None
    }

    pub fn set_index(&mut self, index: Option<usize>) {
        self.index = index;
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_next() {
        let mut reader = Reader::new("Some contents".to_string());
        assert_eq!(reader.next(), Some('S'));
        assert_eq!(reader.next(), Some('o'));
    }

    #[test]
    fn test_reader_peek() {
        let mut reader = Reader::new("Some contents".to_string());
        assert_eq!(reader.next(), Some('S'));
        assert_eq!(reader.peek(), Some('o'));
        assert_eq!(reader.peek(), Some('o'));
        assert_eq!(reader.next(), Some('o'));
        assert_eq!(reader.peek(), Some('m'));
    }
}
