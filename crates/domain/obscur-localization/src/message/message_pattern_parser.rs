use crate::plural::PluralCategory;

use super::{MessagePattern, MessagePatternError, pattern_part::PatternPart};

fn is_utf8_continuation(byte: u8) -> bool {
    byte & 0b1100_0000 == 0b1000_0000
}

#[doc = include_str!("../../docs/api/message/message_pattern.doc/Parser.md")]
pub(super) struct MessagePatternParser<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> MessagePatternParser<'a> {
    pub(super) fn parse(source: &'a str) -> Result<MessagePattern, MessagePatternError> {
        let mut parser = Self {
            input: source.as_bytes(),
            at: 0,
        };
        let pattern = parser.pattern(false)?;
        if parser.at != parser.input.len() {
            // Only an unmatched `}` stops the top-level scan early.
            return Err(MessagePatternError::UnexpectedBrace);
        }
        Ok(pattern)
    }

    #[doc = include_str!("../../docs/api/message/message_pattern.doc/pattern.md")]
    fn pattern(&mut self, in_plural_arm: bool) -> Result<MessagePattern, MessagePatternError> {
        let mut parts = Vec::new();
        let mut literal = String::new();
        while self.at < self.input.len() {
            match self.input[self.at] {
                b'}' => break,
                b'{' => {
                    if !literal.is_empty() {
                        parts.push(PatternPart::Literal(std::mem::take(&mut literal).into()));
                    }
                    self.at += 1;
                    parts.push(self.placeholder(in_plural_arm)?);
                }
                b'#' if in_plural_arm => {
                    if !literal.is_empty() {
                        parts.push(PatternPart::Literal(std::mem::take(&mut literal).into()));
                    }
                    self.at += 1;
                    parts.push(PatternPart::Number);
                }
                _ => {
                    let start = self.at;
                    self.at += 1;
                    while self.at < self.input.len() && is_utf8_continuation(self.input[self.at]) {
                        self.at += 1;
                    }
                    // Both ends land on char boundaries of the original `&str`.
                    literal.push_str(
                        std::str::from_utf8(&self.input[start..self.at])
                            .map_err(|_| MessagePatternError::UnexpectedBrace)?,
                    );
                }
            }
        }
        if !literal.is_empty() {
            parts.push(PatternPart::Literal(literal.into()));
        }
        Ok(MessagePattern::from_parts(parts))
    }

    #[doc = include_str!("../../docs/api/message/message_pattern.doc/placeholder.md")]
    fn placeholder(&mut self, in_plural_arm: bool) -> Result<PatternPart, MessagePatternError> {
        let name = self.token(|b| b == b',' || b == b'}')?;
        if name.is_empty() {
            return Err(MessagePatternError::EmptyArgumentName);
        }
        match self.peek() {
            Some(b'}') => {
                self.at += 1;
                Ok(PatternPart::Argument(name.into()))
            }
            Some(b',') => {
                self.at += 1;
                let selector = self.token(|b| b == b',' || b == b'}')?;
                if self.peek() != Some(b',') {
                    return Err(MessagePatternError::NoArms);
                }
                self.at += 1;
                match selector {
                    // A `plural`'s own arms enable `#` regardless of context.
                    "plural" => self.plural_arms(name),
                    // A `select`'s arms inherit it: see this function's docs.
                    "select" => self.select_arms(name, in_plural_arm),
                    other => Err(MessagePatternError::UnknownSelector(other.into())),
                }
            }
            _ => Err(MessagePatternError::UnclosedPlaceholder),
        }
    }

    /// Parses `<arm>…}` for a `plural` placeholder.
    fn plural_arms(&mut self, name: &str) -> Result<PatternPart, MessagePatternError> {
        let arms = self.arms(true)?;
        let mut exact = Vec::new();
        let mut categories = Vec::new();
        let mut other = None;
        for (arm, pattern) in arms {
            if let Some(digits) = arm.strip_prefix('=') {
                let value = digits
                    .parse::<u64>()
                    .map_err(|_| MessagePatternError::UnknownPluralArm(arm.clone()))?;
                exact.push((value, pattern));
            } else if &*arm == "other" {
                other = Some(pattern);
            } else if let Some(category) = PluralCategory::from_keyword(&arm) {
                categories.push((category, pattern));
            } else {
                return Err(MessagePatternError::UnknownPluralArm(arm));
            }
        }
        Ok(PatternPart::Plural {
            name: name.into(),
            exact,
            categories,
            other: other.ok_or(MessagePatternError::MissingOtherArm)?,
        })
    }

    /// Parses `<arm>…}` for a `select` placeholder, whose arms inherit
    /// `in_plural_arm` from the context the placeholder appears in.
    fn select_arms(
        &mut self,
        name: &str,
        in_plural_arm: bool,
    ) -> Result<PatternPart, MessagePatternError> {
        let mut arms = Vec::new();
        let mut other = None;
        for (arm, pattern) in self.arms(in_plural_arm)? {
            if &*arm == "other" {
                other = Some(pattern);
            } else {
                arms.push((arm, pattern));
            }
        }
        Ok(PatternPart::Select {
            name: name.into(),
            arms,
            other: other.ok_or(MessagePatternError::MissingOtherArm)?,
        })
    }

    /// Parses `name{body}` arms up to the placeholder's closing `}`.
    fn arms(
        &mut self,
        in_plural: bool,
    ) -> Result<Vec<(Box<str>, MessagePattern)>, MessagePatternError> {
        let mut arms = Vec::new();
        loop {
            self.skip_space();
            match self.peek() {
                Some(b'}') => {
                    self.at += 1;
                    break;
                }
                None => return Err(MessagePatternError::UnclosedPlaceholder),
                _ => {}
            }
            let arm = self.token(|b| b == b'{')?.to_owned();
            if arm.is_empty() {
                return Err(MessagePatternError::EmptyArgumentName);
            }
            if self.peek() != Some(b'{') {
                return Err(MessagePatternError::UnclosedPlaceholder);
            }
            self.at += 1;
            let body = self.pattern(in_plural)?;
            if self.peek() != Some(b'}') {
                return Err(MessagePatternError::UnclosedPlaceholder);
            }
            self.at += 1;
            arms.push((arm.into_boxed_str(), body));
        }
        if arms.is_empty() {
            return Err(MessagePatternError::NoArms);
        }
        Ok(arms)
    }

    /// Reads a trimmed token up to the first byte `stop` accepts, without
    /// consuming it.
    fn token(&mut self, stop: impl Fn(u8) -> bool) -> Result<&'a str, MessagePatternError> {
        let start = self.at;
        while self.at < self.input.len() && !stop(self.input[self.at]) {
            self.at += 1;
        }
        // The scan stops only on ASCII, so both ends are char boundaries.
        std::str::from_utf8(&self.input[start..self.at])
            .map(str::trim)
            .map_err(|_| MessagePatternError::UnexpectedBrace)
    }

    /// Skips ASCII whitespace between arms.
    fn skip_space(&mut self) {
        while self.at < self.input.len() && self.input[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }

    /// Returns the byte at the cursor, if any.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
    }
}
