use libc::{c_char, c_double, c_int};

pub(crate) fn value_int(number: f64) -> i32 {
    if number >= i32::MAX as f64 {
        i32::MAX
    } else if number <= i32::MIN as f64 {
        i32::MIN
    } else {
        number as i32
    }
}

pub(crate) fn compare_double(a: f64, b: f64) -> bool {
    let max_val = a.abs().max(b.abs());
    (a - b).abs() <= max_val * f64::EPSILON
}

pub(crate) fn parse_strtod(text: &[u8]) -> Option<(f64, usize)> {
    let mut owned = Vec::with_capacity(text.len() + 1);
    owned.extend_from_slice(text);
    owned.push(0);
    let mut end: *mut c_char = std::ptr::null_mut();
    let value = unsafe { libc::strtod(owned.as_ptr() as *const c_char, &mut end) };
    if end.is_null() || end == owned.as_ptr() as *mut c_char {
        return None;
    }
    let consumed = unsafe { end.offset_from(owned.as_ptr() as *const c_char) };
    if consumed < 0 {
        return None;
    }
    Some((value, consumed as usize))
}

pub(crate) fn format_number(valuedouble: f64, valueint: i32) -> Option<String> {
    let mut buf = [0u8; 26];
    let length = if valuedouble.is_nan() || valuedouble.is_infinite() {
        write_fmt(&mut buf, b"null\0", FormatArg::None)
    } else if valuedouble == valueint as f64 {
        write_fmt(&mut buf, b"%d\0", FormatArg::Int(valueint as c_int))
    } else {
        let mut length = write_fmt(&mut buf, b"%1.15g\0", FormatArg::Float(valuedouble))?;
        if length < 0 || length > 25 {
            return None;
        }
        let mut test: c_double = 0.0;
        let scanned = unsafe {
            libc::sscanf(
                buf.as_ptr() as *const c_char,
                b"%lg\0".as_ptr() as *const c_char,
                &mut test,
            )
        };
        if scanned != 1 || !compare_double(test, valuedouble) {
            length = write_fmt(&mut buf, b"%1.17g\0", FormatArg::Float(valuedouble))?;
        }
        Some(length)
    }?;
    if length < 0 || length > (buf.len() as c_int) - 1 {
        return None;
    }
    let mut out = Vec::with_capacity(length as usize);
    for i in 0..length as usize {
        let byte = buf[i];
        if byte == b',' {
            out.push(b'.');
        } else {
            out.push(byte);
        }
    }
    String::from_utf8(out).ok()
}

enum FormatArg {
    None,
    Int(c_int),
    Float(f64),
}

fn write_fmt(buf: &mut [u8], fmt: &[u8], arg: FormatArg) -> Option<c_int> {
    let length = unsafe {
        match arg {
            FormatArg::None => libc::snprintf(
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                fmt.as_ptr() as *const c_char,
            ),
            FormatArg::Int(value) => libc::snprintf(
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                fmt.as_ptr() as *const c_char,
                value,
            ),
            FormatArg::Float(value) => libc::snprintf(
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                fmt.as_ptr() as *const c_char,
                value,
            ),
        }
    };
    if length < 0 {
        None
    } else {
        Some(length)
    }
}
