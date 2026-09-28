use crate::number::format_number;
use crate::value::{c_bytes, Json, Value, NESTING_LIMIT};

pub fn print(item: &Json) -> Option<String> {
    render(item, true)
}

pub fn print_unformatted(item: &Json) -> Option<String> {
    render(item, false)
}

pub fn print_buffered(item: &Json, prebuffer: i32, format: bool) -> Option<String> {
    if prebuffer < 0 {
        return None;
    }
    render(item, format)
}

pub fn print_preallocated(item: &Json, buffer: &mut [u8], format: bool) -> bool {
    let Some(text) = render(item, format) else {
        return false;
    };
    let bytes = text.as_bytes();
    if bytes.len() + 1 > buffer.len() {
        return false;
    }
    buffer[..bytes.len()].copy_from_slice(bytes);
    buffer[bytes.len()] = 0;
    true
}

fn render(item: &Json, format: bool) -> Option<String> {
    let mut out = Vec::new();
    if !print_value(item, &mut out, format, 0) {
        return None;
    }
    String::from_utf8(out).ok()
}

fn print_value(item: &Json, out: &mut Vec<u8>, format: bool, depth: usize) -> bool {
    match &item.value {
        Value::Null => {
            out.extend_from_slice(b"null");
            true
        }
        Value::Bool(false) => {
            out.extend_from_slice(b"false");
            true
        }
        Value::Bool(true) => {
            out.extend_from_slice(b"true");
            true
        }
        Value::Number {
            valuedouble,
            valueint,
        } => match format_number(*valuedouble, *valueint) {
            Some(text) => {
                out.extend_from_slice(text.as_bytes());
                true
            }
            None => false,
        },
        Value::Raw(text) => {
            out.extend_from_slice(c_bytes(text));
            true
        }
        Value::String(text) => {
            print_string(out, text);
            true
        }
        Value::Array(items) => print_array(items, out, format, depth),
        Value::Object(items) => print_object(items, out, format, depth),
    }
}

fn print_string(out: &mut Vec<u8>, text: &str) {
    let input = c_bytes(text);
    out.push(b'"');
    for byte in input {
        if *byte > 31 && *byte != b'"' && *byte != b'\\' {
            out.push(*byte);
            continue;
        }
        out.push(b'\\');
        match *byte {
            b'\\' => out.push(b'\\'),
            b'"' => out.push(b'"'),
            8 => out.push(b'b'),
            12 => out.push(b'f'),
            b'\n' => out.push(b'n'),
            b'\r' => out.push(b'r'),
            b'\t' => out.push(b't'),
            other => {
                let escaped = format!("u{other:04x}");
                out.extend_from_slice(escaped.as_bytes());
            }
        }
    }
    out.push(b'"');
}

fn print_array(items: &[Json], out: &mut Vec<u8>, format: bool, depth: usize) -> bool {
    if depth >= NESTING_LIMIT {
        return false;
    }
    let depth = depth + 1;
    out.push(b'[');
    for (index, child) in items.iter().enumerate() {
        if !print_value(child, out, format, depth) {
            return false;
        }
        if index + 1 != items.len() {
            out.push(b',');
            if format {
                out.push(b' ');
            }
        }
    }
    out.push(b']');
    true
}

fn print_object(items: &[Json], out: &mut Vec<u8>, format: bool, depth: usize) -> bool {
    if depth >= NESTING_LIMIT {
        return false;
    }
    let depth = depth + 1;
    out.push(b'{');
    if format {
        out.push(b'\n');
    }
    for (index, child) in items.iter().enumerate() {
        if format {
            out.resize(out.len() + depth, b'\t');
        }
        let name = child
            .key
            .as_ref()
            .map(|key| key.name.as_str())
            .unwrap_or("");
        print_string(out, name);
        out.push(b':');
        if format {
            out.push(b'\t');
        }
        if !print_value(child, out, format, depth) {
            return false;
        }
        if index + 1 != items.len() {
            out.push(b',');
        }
        if format {
            out.push(b'\n');
        }
    }
    if format {
        let tabs = depth.saturating_sub(1);
        out.resize(out.len() + tabs, b'\t');
    }
    out.push(b'}');
    true
}
