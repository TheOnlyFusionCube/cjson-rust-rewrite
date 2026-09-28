pub fn minify(json: &mut String) {
    let input = std::mem::take(json);
    let bytes = input.into_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() && bytes[index] != 0 {
        match bytes[index] {
            b' ' | b'\t' | b'\r' | b'\n' => index += 1,
            b'/' => {
                if bytes.get(index + 1) == Some(&b'/') {
                    index += 2;
                    while index < bytes.len() && bytes[index] != 0 {
                        if bytes[index] == b'\n' {
                            index += 1;
                            break;
                        }
                        index += 1;
                    }
                } else if bytes.get(index + 1) == Some(&b'*') {
                    index += 2;
                    while index < bytes.len() && bytes[index] != 0 {
                        if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                            index += 2;
                            break;
                        }
                        index += 1;
                    }
                } else {
                    index += 1;
                }
            }
            b'"' => {
                out.push(b'"');
                index += 1;
                while index < bytes.len() && bytes[index] != 0 {
                    out.push(bytes[index]);
                    if bytes[index] == b'"' {
                        index += 1;
                        break;
                    } else if bytes[index] == b'\\' && bytes.get(index + 1) == Some(&b'"') {
                        index += 1;
                        if index < bytes.len() {
                            out.push(bytes[index]);
                        }
                        index += 1;
                    } else {
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    *json = String::from_utf8(out)
        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned());
}
