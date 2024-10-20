

pub const fn tokenise(text: &str) -> TokenStream {
    TokenStream { text, byte: 0 }
}

#[derive(Debug)]
pub struct TokenStream<'a> {
    text: &'a str,
    byte: usize,
}

#[derive(Debug)]
pub enum Token<'a> {
    Literal(&'a str),
    LParen,
    RParen,
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.text.as_bytes();
        match &bytes[self.byte..] {
            &[] => None,
            &[b'(', ..] => {
                self.byte += 1;
                Some(Token::LParen)
            }
            &[b')', ..] => {
                self.byte += 1;
                Some(Token::RParen)
            }
            text @ &[w, ..] if w.is_ascii_whitespace() => {
                // strip leading whitespace
                let n_advanced = text
                    .iter()
                    .position(|b| !b.is_ascii_whitespace())
                    .unwrap_or(0);
                self.byte += n_advanced;
                self.next()
            }
            text => {
                // search until paren or whitespace
                let token_len = text
                    .iter()
                    .position(|b| b.is_ascii_whitespace() || *b == b')' || *b == b'(')
                    .unwrap_or(text.len());
                self.byte += token_len;
                // slow, can be unchecked:
                let token_text = std::str::from_utf8(&text[..token_len]).unwrap();
                Some(Token::Literal(token_text))
            }
        }
    }
}
