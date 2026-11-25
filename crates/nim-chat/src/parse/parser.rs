use crate::{ChatMessageContent, ChatMessageContentPart, Cursor, ParseMode};

pub const EOF: char = '\x04';

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    mode: ParseMode,
    cursor: Cursor<'a>,
    content: Option<ChatMessageContent>,
}

impl<'a> Parser<'a> {
    pub fn from_cursor(cursor: Cursor<'a>, mode: ParseMode) -> Self {
        Self {
            mode,
            cursor,
            content: None,
        }
    }

    pub fn from_src(src: &'a str, mode: ParseMode) -> Self {
        Self {
            mode,
            cursor: Cursor::new(src),
            content: None,
        }
    }

    pub fn mode(&self) -> &ParseMode {
        &self.mode
    }

    pub fn remaining(&self) -> &'a str {
        self.cursor.src()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_none()
    }

    pub fn len(&self) -> usize {
        match &self.content {
            None => 0,
            Some(ChatMessageContent::String(_)) => 1,
            Some(ChatMessageContent::Parts(parts)) => parts.len(),
        }
    }

    pub fn is_eof(&self) -> bool {
        self.cursor.is_empty()
    }

    pub fn curr(&self) -> char {
        match self.cursor.first() {
            None => EOF,
            Some(v) => v,
        }
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.cursor.starts_with(s)
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_n(1)
    }

    pub fn peek_n(&self, offset: usize) -> Option<char> {
        self.cursor.src().chars().nth(offset)
    }

    pub fn seek(&self, c: char) -> Option<usize> {
        let cursor = self.cursor.skip_while(|curr| curr != c);

        if cursor.is_empty() {
            None
        } else {
            Some(cursor.offset() as usize)
        }
    }

    #[track_caller]
    pub fn error(&self, message: impl std::fmt::Display) -> nim_error::Diagnostic {
        nim_error::Diagnostic::message(message)
    }

    pub fn push(&mut self, part: ChatMessageContentPart) {
        match &mut self.content {
            None => {
                self.content = Some(ChatMessageContent::Parts(vec![part]));
            }
            Some(ChatMessageContent::String(v)) => {
                self.content = Some(ChatMessageContent::Parts(vec![
                    ChatMessageContentPart::Text { text: v.clone() },
                    part,
                ]));
            }
            Some(ChatMessageContent::Parts(parts)) => {
                parts.push(part);
            }
        }
    }

    pub fn push_str(&mut self, s: &str) {
        match &mut self.content {
            None => {
                self.content = Some(ChatMessageContent::String(s.to_string()));
            }
            Some(ChatMessageContent::String(v)) => {
                self.content = Some(ChatMessageContent::Parts(vec![
                    ChatMessageContentPart::Text { text: v.clone() },
                    ChatMessageContentPart::Text { text: s.to_string() },
                ]));
            }
            Some(ChatMessageContent::Parts(parts)) => {
                parts.push(ChatMessageContentPart::Text { text: s.to_string() });
            }
        }
    }

    pub fn advance(&mut self) -> &mut Self {
        self.cursor = self.cursor.advance();
        self
    }

    pub fn advance_by(&mut self, n: usize) -> &mut Self {
        self.cursor = self.cursor.advance_by(n);
        self
    }

    pub fn skip_while(&mut self, p: impl FnMut(char) -> bool) -> &mut Self {
        self.cursor = self.cursor.skip_while(p);
        self
    }
}
