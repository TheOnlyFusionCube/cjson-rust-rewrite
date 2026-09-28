use std::cell::Cell;

use crate::number::{parse_strtod, value_int};
use crate::value::{Json, Value, NESTING_LIMIT};

thread_local! {
    static ERROR_OFFSET: Cell<Option<usize>> = const { Cell::new(None) };
}

pub struct Parsed {
    pub item: Option<Json>,
    pub end: usize,
}

struct Buf<'a> {
    content: &'a [u8],
    length: usize,
    offset: usize,
    depth: usize,
}

impl<'a> Buf<'a> {
    fn can_read(&self, size: usize) -> bool {
        self.offset
            .checked_add(size)
            .is_some_and(|end| end <= self.length)
    }

    fn can_access(&self, index: usize) -> bool {
        self.offset
            .checked_add(index)
            .is_some_and(|pos| pos < self.length)
    }

    fn at(&self, index: usize) -> u8 {
        let Some(pos) = self.offset.checked_add(index) else {
            return 0;
        };
        if pos < self.length && pos < self.content.len() {
            self.content[pos]
        } else {
            0
        }
    }

    fn abs(&self, pos: usize) -> u8 {
        if pos < self.length && pos < self.content.len() {
            self.content[pos]
        } else {
            0
        }
    }
}

fn set_error(offset: Option<usize>) {
    ERROR_OFFSET.with(|slot| slot.set(offset));
}

pub fn get_error_ptr() -> Option<usize> {
    ERROR_OFFSET.with(|slot| slot.get())
}

pub fn parse(value: &str) -> Option<Json> {
    parse_with_opts(value, false).item
}

pub fn parse_with_length(value: &[u8], buffer_length: usize) -> Option<Json> {
    parse_with_length_opts(value, buffer_length, false).item
}

pub fn parse_with_opts(value: &str, require_null_terminated: bool) -> Parsed {
    parse_with_length_opts(value.as_bytes(), value.len() + 1, require_null_terminated)
}

pub fn parse_with_length_opts(
    value: &[u8],
    buffer_length: usize,
    require_null_terminated: bool,
) -> Parsed {
    set_error(None);
    if buffer_length == 0 {
        set_error(Some(0));
        return Parsed { item: None, end: 0 };
    }
    let length = buffer_length.min(value.len().saturating_add(1));
    let mut buf = Buf {
        content: value,
        length,
        offset: 0,
        depth: 0,
    };
    skip_bom(&mut buf);
    skip_ws(&mut buf);
    match parse_value(&mut buf) {
        Some(item) => {
            if require_null_terminated {
                skip_ws(&mut buf);
                if buf.offset >= buf.length || buf.at(0) != 0 {
                    let pos = error_pos(&buf);
                    set_error(Some(pos));
                    return Parsed {
                        item: None,
                        end: pos,
                    };
                }
            }
            Parsed {
                item: Some(item),
                end: buf.offset,
            }
        }
        None => {
            let pos = error_pos(&buf);
            set_error(Some(pos));
            Parsed {
                item: None,
                end: pos,
            }
        }
    }
}

fn error_pos(buf: &Buf) -> usize {
    if buf.offset < buf.length {
        buf.offset
    } else if buf.length > 0 {
        buf.length - 1
    } else {
        0
    }
}

fn skip_bom(buf: &mut Buf) {
    if buf.offset != 0 {
        return;
    }
    if buf.can_access(4) && buf.at(0) == 0xEF && buf.at(1) == 0xBB && buf.at(2) == 0xBF {
        buf.offset += 3;
    }
}

fn skip_ws(buf: &mut Buf) {
    if !buf.can_access(0) {
        return;
    }
    while buf.can_access(0) && buf.at(0) <= 32 {
        buf.offset += 1;
    }
    if buf.offset == buf.length && buf.offset > 0 {
        buf.offset -= 1;
    }
}

fn parse_value(buf: &mut Buf) -> Option<Json> {
    if buf.starts_with(b"null") {
        buf.offset += 4;
        return Some(node(Value::Null));
    }
    if buf.starts_with(b"false") {
        buf.offset += 5;
        return Some(node(Value::Bool(false)));
    }
    if buf.starts_with(b"true") {
        buf.offset += 4;
        return Some(node(Value::Bool(true)));
    }
    if buf.can_access(0) && buf.at(0) == b'"' {
        let text = parse_string(buf)?;
        return Some(node(Value::String(text)));
    }
    if buf.can_access(0) && (buf.at(0) == b'-' || buf.at(0).is_ascii_digit()) {
        return parse_number(buf);
    }
    if buf.can_access(0) && buf.at(0) == b'[' {
        let items = parse_array(buf)?;
        return Some(node(Value::Array(items)));
    }
    if buf.can_access(0) && buf.at(0) == b'{' {
        let items = parse_object(buf)?;
        return Some(node(Value::Object(items)));
    }
    None
}

impl<'a> Buf<'a> {
    fn starts_with(&self, literal: &[u8]) -> bool {
        if !self.can_read(literal.len()) {
            return false;
        }
        literal
            .iter()
            .enumerate()
            .all(|(index, byte)| self.at(index) == *byte)
    }
}

fn node(value: Value) -> Json {
    Json {
        value,
        key: None,
        reference: false,
    }
}

fn parse_number(buf: &mut Buf) -> Option<Json> {
    let mut length = 0usize;
    while buf.can_access(length) {
        match buf.at(length) {
            b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' | b'.' => length += 1,
            _ => break,
        }
    }
    let start = buf.offset;
    let end = start + length;
    let slice_end = end.min(buf.content.len());
    let text = if start < buf.content.len() {
        &buf.content[start..slice_end]
    } else {
        b""
    };
    let (number, consumed) = parse_strtod(text)?;
    if consumed == 0 {
        return None;
    }
    buf.offset += consumed;
    Some(node(Value::Number {
        valuedouble: number,
        valueint: value_int(number),
    }))
}

fn parse_string(buf: &mut Buf) -> Option<String> {
    if !buf.can_access(0) || buf.at(0) != b'"' {
        return None;
    }
    let mut cursor = buf.offset + 1;
    while cursor < buf.length && buf.abs(cursor) != b'"' {
        if buf.abs(cursor) == b'\\' {
            if cursor + 1 >= buf.length {
                buf.offset = cursor;
                return None;
            }
            cursor += 1;
        }
        cursor += 1;
    }
    if cursor >= buf.length || buf.abs(cursor) != b'"' {
        buf.offset = cursor.min(buf.length.saturating_sub(1));
        return None;
    }
    let quote = cursor;
    let mut raw = Vec::new();
    let mut index = buf.offset + 1;
    while index < quote {
        if buf.abs(index) != b'\\' {
            raw.push(buf.abs(index));
            index += 1;
            continue;
        }
        if quote - index < 1 {
            buf.offset = index;
            return None;
        }
        let next = buf.abs(index + 1);
        match next {
            b'b' => raw.push(8),
            b'f' => raw.push(12),
            b'n' => raw.push(b'\n'),
            b'r' => raw.push(b'\r'),
            b't' => raw.push(b'\t'),
            b'"' | b'\\' | b'/' => raw.push(next),
            b'u' => {
                let available = &window(buf, index, quote);
                let (sequence, bytes) = utf16_literal_to_utf8(available)?;
                raw.extend_from_slice(&bytes);
                index += sequence;
                continue;
            }
            _ => {
                buf.offset = index;
                return None;
            }
        }
        index += 2;
    }
    buf.offset = quote + 1;
    Some(bytes_to_string(raw))
}

fn window(buf: &Buf, start: usize, end: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = start;
    while index < end {
        out.push(buf.abs(index));
        index += 1;
    }
    out
}

fn bytes_to_string(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes)
        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned())
}

fn utf16_literal_to_utf8(input: &[u8]) -> Option<(usize, Vec<u8>)> {
    if input.len() < 6 {
        return None;
    }
    let first_code = parse_hex4(&input[2..6])?;
    if (0xDC00..=0xDFFF).contains(&first_code) {
        return None;
    }
    let (sequence_length, codepoint) = if (0xD800..=0xDBFF).contains(&first_code) {
        if input.len() < 12 || input[6] != b'\\' || input[7] != b'u' {
            return None;
        }
        let second_code = parse_hex4(&input[8..12])?;
        if !(0xDC00..=0xDFFF).contains(&second_code) {
            return None;
        }
        let point = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
        (12usize, point)
    } else {
        (6usize, first_code)
    };
    Some((sequence_length, encode_utf8(codepoint)?))
}

fn parse_hex4(input: &[u8]) -> Option<u32> {
    if input.len() < 4 {
        return None;
    }
    let mut value = 0u32;
    for (index, byte) in input.iter().copied().take(4).enumerate() {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'A'..=b'F' => byte - b'A' + 10,
            b'a'..=b'f' => byte - b'a' + 10,
            _ => return Some(0),
        };
        value += u32::from(digit);
        if index < 3 {
            value <<= 4;
        }
    }
    Some(value)
}

fn encode_utf8(mut codepoint: u32) -> Option<Vec<u8>> {
    let (length, first_mark) = if codepoint < 0x80 {
        (1usize, 0u32)
    } else if codepoint < 0x800 {
        (2, 0xC0)
    } else if codepoint < 0x10000 {
        (3, 0xE0)
    } else if codepoint <= 0x10FFFF {
        (4, 0xF0)
    } else {
        return None;
    };
    let mut out = vec![0u8; length];
    for position in (1..length).rev() {
        out[position] = ((codepoint | 0x80) & 0xBF) as u8;
        codepoint >>= 6;
    }
    out[0] = if length > 1 {
        ((codepoint | first_mark) & 0xFF) as u8
    } else {
        (codepoint & 0x7F) as u8
    };
    Some(out)
}

fn parse_array(buf: &mut Buf) -> Option<Vec<Json>> {
    if buf.depth >= NESTING_LIMIT {
        return None;
    }
    buf.depth += 1;
    if !buf.can_access(0) || buf.at(0) != b'[' {
        return None;
    }
    buf.offset += 1;
    skip_ws(buf);
    if buf.can_access(0) && buf.at(0) == b']' {
        buf.depth -= 1;
        buf.offset += 1;
        return Some(Vec::new());
    }
    if !buf.can_access(0) {
        if buf.offset > 0 {
            buf.offset -= 1;
        }
        return None;
    }
    if buf.offset == 0 {
        return None;
    }
    buf.offset -= 1;
    let mut items = Vec::new();
    loop {
        buf.offset += 1;
        skip_ws(buf);
        items.push(parse_value(buf)?);
        skip_ws(buf);
        if !(buf.can_access(0) && buf.at(0) == b',') {
            break;
        }
    }
    if !buf.can_access(0) || buf.at(0) != b']' {
        return None;
    }
    buf.depth -= 1;
    buf.offset += 1;
    Some(items)
}

fn parse_object(buf: &mut Buf) -> Option<Vec<Json>> {
    if buf.depth >= NESTING_LIMIT {
        return None;
    }
    buf.depth += 1;
    if !buf.can_access(0) || buf.at(0) != b'{' {
        return None;
    }
    buf.offset += 1;
    skip_ws(buf);
    if buf.can_access(0) && buf.at(0) == b'}' {
        buf.depth -= 1;
        buf.offset += 1;
        return Some(Vec::new());
    }
    if !buf.can_access(0) {
        if buf.offset > 0 {
            buf.offset -= 1;
        }
        return None;
    }
    if buf.offset == 0 {
        return None;
    }
    buf.offset -= 1;
    let mut items = Vec::new();
    loop {
        if !buf.can_access(1) {
            return None;
        }
        buf.offset += 1;
        skip_ws(buf);
        let name = parse_string(buf)?;
        skip_ws(buf);
        if !buf.can_access(0) || buf.at(0) != b':' {
            return None;
        }
        buf.offset += 1;
        skip_ws(buf);
        let mut child = parse_value(buf)?;
        child.key = Some(crate::value::Key {
            name,
            constant: false,
        });
        items.push(child);
        skip_ws(buf);
        if !(buf.can_access(0) && buf.at(0) == b',') {
            break;
        }
    }
    if !buf.can_access(0) || buf.at(0) != b'}' {
        return None;
    }
    buf.depth -= 1;
    buf.offset += 1;
    Some(items)
}
