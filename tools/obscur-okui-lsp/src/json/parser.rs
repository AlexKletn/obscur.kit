#![doc = include_str!("parser.doc/_module.md")]

use crate::json::{JsonError, JsonValue};

#[doc = include_str!("parser.doc/parse.md")]
pub fn parse(input: &str) -> Result<JsonValue, JsonError> {
    let chars: Vec<char> = input.chars().collect();
    let mut parser = ParseState {
        chars: &chars,
        pos: 0,
    };
    parser.skip_whitespace();
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.pos != parser.chars.len() {
        return Err(parser.error("unexpected trailing characters"));
    }
    Ok(value)
}

#[doc = include_str!("parser.doc/ParseState.md")]
struct ParseState<'a> {
    chars: &'a [char],
    pos: usize,
}

impl ParseState<'_> {
    fn error(&self, message: &str) -> JsonError {
        JsonError::new(message, self.pos)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<JsonValue, JsonError> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => self.string().map(JsonValue::String),
            Some('t' | 'f') => self.boolean(),
            Some('n') => self.null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            Some(_) => Err(self.error("unexpected character")),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn object(&mut self) -> Result<JsonValue, JsonError> {
        self.bump(); // '{'
        let mut entries = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(self.error("expected object key string"));
            }
            let key = self.string()?;
            self.skip_whitespace();
            if self.bump() != Some(':') {
                return Err(self.error("expected ':' after object key"));
            }
            self.skip_whitespace();
            let value = self.value()?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.bump() {
                Some(',') => {}
                Some('}') => return Ok(JsonValue::Object(entries)),
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue, JsonError> {
        self.bump(); // '['
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value()?);
            self.skip_whitespace();
            match self.bump() {
                Some(',') => {}
                Some(']') => return Ok(JsonValue::Array(items)),
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.bump(); // opening '"'
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.error("unterminated string")),
                Some('"') => return Ok(out),
                Some('\\') => self.escape(&mut out)?,
                Some(c) if (c as u32) < 0x20 => {
                    return Err(self.error("control character in string"));
                }
                Some(c) => out.push(c),
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), JsonError> {
        match self.bump() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('b') => out.push('\u{08}'),
            Some('f') => out.push('\u{0c}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => self.unicode_escape(out)?,
            _ => return Err(self.error("invalid escape sequence")),
        }
        Ok(())
    }

    fn unicode_escape(&mut self, out: &mut String) -> Result<(), JsonError> {
        let first = self.hex4()?;
        // High surrogate: must be followed by a `\uXXXX` low surrogate.
        if (0xD800..=0xDBFF).contains(&first) {
            if self.bump() != Some('\\') || self.bump() != Some('u') {
                return Err(self.error("expected low surrogate after high surrogate"));
            }
            let second = self.hex4()?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(self.error("invalid low surrogate"));
            }
            let code = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
            match char::from_u32(code) {
                Some(c) => out.push(c),
                None => return Err(self.error("invalid surrogate pair")),
            }
            return Ok(());
        }
        match char::from_u32(first) {
            Some(c) => out.push(c),
            None => return Err(self.error("invalid unicode escape")),
        }
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, JsonError> {
        let mut code = 0u32;
        for _ in 0..4 {
            let digit = self
                .bump()
                .and_then(|c| c.to_digit(16))
                .ok_or_else(|| self.error("invalid \\u escape"))?;
            code = code * 16 + digit;
        }
        Ok(code)
    }

    fn boolean(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn null(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume("null") {
            Ok(JsonValue::Null)
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn consume(&mut self, word: &str) -> bool {
        let end = self.pos + word.len();
        if end <= self.chars.len() && self.chars[self.pos..end].iter().copied().eq(word.chars()) {
            self.pos = end;
            true
        } else {
            false
        }
    }

    fn number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.bump();
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.bump();
            if matches!(self.peek(), Some('+' | '-')) {
                self.bump();
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| JsonError::new("invalid number", start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
        assert_eq!(parse("  -12.5e2 ").unwrap(), JsonValue::Number(-1250.0));
    }

    #[test]
    fn parses_object_and_array() {
        let value = parse("{\"a\":[1,2],\"b\":{\"c\":true}}").unwrap();
        assert_eq!(
            value.get("a").and_then(JsonValue::as_array).unwrap().len(),
            2
        );
        assert_eq!(
            value.get("b").and_then(|b| b.get("c")),
            Some(&JsonValue::Bool(true))
        );
    }

    #[test]
    fn parses_string_escapes_including_surrogates() {
        let value = parse("\"a\\n\\t\\\"\\u0041\\uD83D\\uDE00\"").unwrap();
        assert_eq!(value.as_str(), Some("a\n\t\"A\u{1F600}"));
    }

    #[test]
    fn rejects_trailing_and_malformed_input() {
        assert!(parse("{}extra").is_err());
        assert!(parse("[1,]").is_err());
        assert!(parse("\"unterminated").is_err());
    }

    #[test]
    fn round_trips_through_serializer() {
        let source = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true,\"n\":[1,2,3]}}";
        let parsed = parse(source).unwrap();
        assert_eq!(parsed.to_string(), source);
    }

    #[test]
    fn round_trips_unicode_and_escapes() {
        let original = JsonValue::string("line1\nline2\t\"quoted\" \u{1F600} café");
        let text = original.to_string();
        assert_eq!(parse(&text).unwrap(), original);
    }
}
