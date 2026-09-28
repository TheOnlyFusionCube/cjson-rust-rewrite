use crate::number::{compare_double, value_int};

pub const NESTING_LIMIT: usize = 1000;
pub const CIRCULAR_LIMIT: usize = 10000;

pub const TYPE_INVALID: i32 = 0;
pub const TYPE_FALSE: i32 = 1 << 0;
pub const TYPE_TRUE: i32 = 1 << 1;
pub const TYPE_NULL: i32 = 1 << 2;
pub const TYPE_NUMBER: i32 = 1 << 3;
pub const TYPE_STRING: i32 = 1 << 4;
pub const TYPE_ARRAY: i32 = 1 << 5;
pub const TYPE_OBJECT: i32 = 1 << 6;
pub const TYPE_RAW: i32 = 1 << 7;
pub const TYPE_IS_REFERENCE: i32 = 256;
pub const TYPE_STRING_IS_CONST: i32 = 512;

#[derive(Clone, Debug)]
pub struct Json {
    pub(crate) value: Value,
    pub(crate) key: Option<Key>,
    pub(crate) reference: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Key {
    pub(crate) name: String,
    pub(crate) constant: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum Value {
    Bool(bool),
    Null,
    Number { valuedouble: f64, valueint: i32 },
    String(String),
    Raw(String),
    Array(Vec<Json>),
    Object(Vec<Json>),
}

pub fn c_bytes(text: &str) -> &[u8] {
    match text.as_bytes().iter().position(|byte| *byte == 0) {
        Some(index) => &text.as_bytes()[..index],
        None => text.as_bytes(),
    }
}

pub fn c_prefix(text: &str) -> &str {
    let bytes = c_bytes(text);
    std::str::from_utf8(bytes).unwrap_or("")
}

fn blank(value: Value) -> Json {
    Json {
        value,
        key: None,
        reference: false,
    }
}

pub fn version() -> &'static str {
    "1.7.19"
}

pub fn type_bits(item: &Json) -> i32 {
    let mut bits = match item.value {
        Value::Bool(false) => TYPE_FALSE,
        Value::Bool(true) => TYPE_TRUE,
        Value::Null => TYPE_NULL,
        Value::Number { .. } => TYPE_NUMBER,
        Value::String(_) => TYPE_STRING,
        Value::Raw(_) => TYPE_RAW,
        Value::Array(_) => TYPE_ARRAY,
        Value::Object(_) => TYPE_OBJECT,
    };
    if item.reference {
        bits |= TYPE_IS_REFERENCE;
    }
    if item.key.as_ref().is_some_and(|key| key.constant) {
        bits |= TYPE_STRING_IS_CONST;
    }
    bits
}

pub fn delete(item: Json) {
    drop(item);
}

pub fn key(item: &Json) -> Option<&str> {
    item.key.as_ref().map(|key| c_prefix(&key.name))
}

pub fn is_reference(item: &Json) -> bool {
    item.reference
}

pub fn string_is_const(item: &Json) -> bool {
    item.key.as_ref().is_some_and(|key| key.constant)
}

pub fn is_invalid(_item: &Json) -> bool {
    false
}

pub fn is_false(item: &Json) -> bool {
    matches!(item.value, Value::Bool(false))
}

pub fn is_true(item: &Json) -> bool {
    matches!(item.value, Value::Bool(true))
}

pub fn is_bool(item: &Json) -> bool {
    matches!(item.value, Value::Bool(_))
}

pub fn is_null(item: &Json) -> bool {
    matches!(item.value, Value::Null)
}

pub fn is_number(item: &Json) -> bool {
    matches!(item.value, Value::Number { .. })
}

pub fn is_string(item: &Json) -> bool {
    matches!(item.value, Value::String(_))
}

pub fn is_array(item: &Json) -> bool {
    matches!(item.value, Value::Array(_))
}

pub fn is_object(item: &Json) -> bool {
    matches!(item.value, Value::Object(_))
}

pub fn is_raw(item: &Json) -> bool {
    matches!(item.value, Value::Raw(_))
}

pub fn get_string_value(item: &Json) -> Option<&str> {
    match &item.value {
        Value::String(text) => Some(c_prefix(text)),
        _ => None,
    }
}

pub fn get_number_value(item: &Json) -> f64 {
    match item.value {
        Value::Number { valuedouble, .. } => valuedouble,
        _ => f64::NAN,
    }
}

pub fn create_null() -> Json {
    blank(Value::Null)
}

pub fn create_true() -> Json {
    blank(Value::Bool(true))
}

pub fn create_false() -> Json {
    blank(Value::Bool(false))
}

pub fn create_bool(value: bool) -> Json {
    blank(Value::Bool(value))
}

pub fn create_number(number: f64) -> Json {
    blank(Value::Number {
        valuedouble: number,
        valueint: value_int(number),
    })
}

pub fn create_string(string: &str) -> Json {
    blank(Value::String(string.to_owned()))
}

pub fn create_raw(raw: &str) -> Json {
    blank(Value::Raw(raw.to_owned()))
}

pub fn create_array() -> Json {
    blank(Value::Array(Vec::new()))
}

pub fn create_object() -> Json {
    blank(Value::Object(Vec::new()))
}

pub fn create_string_reference(string: &str) -> Json {
    let mut item = create_string(string);
    item.reference = true;
    item
}

pub fn create_array_reference(children: &[Json]) -> Json {
    let mut item = blank(Value::Array(children.to_vec()));
    item.reference = true;
    item
}

pub fn create_object_reference(children: &[Json]) -> Json {
    let mut item = blank(Value::Object(children.to_vec()));
    item.reference = true;
    item
}

pub fn create_int_array(numbers: &[i32]) -> Json {
    let items = numbers
        .iter()
        .copied()
        .map(|number| create_number(f64::from(number)))
        .collect();
    blank(Value::Array(items))
}

pub fn create_float_array(numbers: &[f32]) -> Json {
    let items = numbers
        .iter()
        .copied()
        .map(|number| create_number(f64::from(number)))
        .collect();
    blank(Value::Array(items))
}

pub fn create_double_array(numbers: &[f64]) -> Json {
    let items = numbers.iter().copied().map(create_number).collect();
    blank(Value::Array(items))
}

pub fn create_string_array(strings: &[&str]) -> Json {
    let items = strings.iter().copied().map(create_string).collect();
    blank(Value::Array(items))
}

fn children(item: &Json) -> &[Json] {
    match &item.value {
        Value::Array(items) | Value::Object(items) => items,
        _ => &[],
    }
}

fn children_mut(item: &mut Json) -> Option<&mut Vec<Json>> {
    match &mut item.value {
        Value::Array(items) | Value::Object(items) => Some(items),
        _ => None,
    }
}

pub fn get_array_size(array: &Json) -> i32 {
    children(array).len() as i32
}

pub fn get_array_item(array: &Json, index: i32) -> Option<&Json> {
    if index < 0 {
        return None;
    }
    children(array).get(index as usize)
}

pub fn get_array_item_mut(array: &mut Json, index: i32) -> Option<&mut Json> {
    if index < 0 {
        return None;
    }
    let index = index as usize;
    children_mut(array)?.get_mut(index)
}

fn key_eq(left: &str, right: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        c_bytes(left) == c_bytes(right)
    } else {
        case_insensitive_eq(c_bytes(left), c_bytes(right))
    }
}

fn case_insensitive_eq(left: &[u8], right: &[u8]) -> bool {
    let mut index = 0;
    loop {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a.to_ascii_lowercase() != b.to_ascii_lowercase() {
            return false;
        }
        if a == 0 {
            return true;
        }
        index += 1;
    }
}

fn find_child(items: &[Json], name: &str, case_sensitive: bool) -> Option<usize> {
    for (index, child) in items.iter().enumerate() {
        let Some(key) = child.key.as_ref() else {
            if case_sensitive {
                return None;
            }
            continue;
        };
        if key_eq(&key.name, name, case_sensitive) {
            return Some(index);
        }
    }
    None
}

pub fn get_object_item<'a>(object: &'a Json, name: &str) -> Option<&'a Json> {
    get_object_item_inner(object, name, false)
}

pub fn get_object_item_case_sensitive<'a>(object: &'a Json, name: &str) -> Option<&'a Json> {
    get_object_item_inner(object, name, true)
}

fn get_object_item_inner<'a>(
    object: &'a Json,
    name: &str,
    case_sensitive: bool,
) -> Option<&'a Json> {
    let items = match &object.value {
        Value::Object(items) | Value::Array(items) => items,
        _ => return None,
    };
    let index = find_child(items, name, case_sensitive)?;
    items.get(index)
}

pub fn get_object_item_mut<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    get_object_item_mut_inner(object, name, false)
}

pub fn get_object_item_case_sensitive_mut<'a>(
    object: &'a mut Json,
    name: &str,
) -> Option<&'a mut Json> {
    get_object_item_mut_inner(object, name, true)
}

fn get_object_item_mut_inner<'a>(
    object: &'a mut Json,
    name: &str,
    case_sensitive: bool,
) -> Option<&'a mut Json> {
    let items = match &mut object.value {
        Value::Object(items) | Value::Array(items) => items,
        _ => return None,
    };
    let index = find_child(items, name, case_sensitive)?;
    items.get_mut(index)
}

pub fn has_object_item(object: &Json, name: &str) -> bool {
    get_object_item(object, name).is_some()
}

pub fn array_for_each(item: &Json) -> impl Iterator<Item = &Json> {
    children(item).iter()
}

pub fn add_item_to_array(array: &mut Json, item: Json) -> bool {
    let Value::Array(items) = &mut array.value else {
        return false;
    };
    items.push(item);
    true
}

pub fn add_item_to_object(object: &mut Json, name: &str, item: Json) -> bool {
    push_object_item(object, name, item, false)
}

pub fn add_item_to_object_cs(object: &mut Json, name: &str, item: Json) -> bool {
    push_object_item(object, name, item, true)
}

fn push_object_item(object: &mut Json, name: &str, mut item: Json, constant_key: bool) -> bool {
    let Value::Object(items) = &mut object.value else {
        return false;
    };
    item.key = Some(Key {
        name: name.to_owned(),
        constant: constant_key,
    });
    items.push(item);
    true
}

fn object_items_mut(object: &mut Json) -> Option<&mut Vec<Json>> {
    match &mut object.value {
        Value::Object(items) => Some(items),
        _ => None,
    }
}

pub fn add_item_reference_to_array(array: &mut Json, item: &Json) -> bool {
    let mut copy = item.clone();
    copy.key = None;
    copy.reference = true;
    add_item_to_array(array, copy)
}

pub fn add_item_reference_to_object(object: &mut Json, name: &str, item: &Json) -> bool {
    let mut copy = item.clone();
    copy.key = None;
    copy.reference = true;
    add_item_to_object(object, name, copy)
}

fn push_named(object: &mut Json, name: &str, item: Json) -> Option<usize> {
    if !add_item_to_object(object, name, item) {
        return None;
    }
    let len = object_items_mut(object)?.len();
    Some(len - 1)
}

pub fn add_null_to_object<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_null())?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_true_to_object<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_true())?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_false_to_object<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_false())?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_bool_to_object<'a>(
    object: &'a mut Json,
    name: &str,
    value: bool,
) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_bool(value))?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_number_to_object<'a>(
    object: &'a mut Json,
    name: &str,
    number: f64,
) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_number(number))?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_string_to_object<'a>(
    object: &'a mut Json,
    name: &str,
    string: &str,
) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_string(string))?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_raw_to_object<'a>(object: &'a mut Json, name: &str, raw: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_raw(raw))?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_object_to_object<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_object())?;
    object_items_mut(object)?.get_mut(index)
}

pub fn add_array_to_object<'a>(object: &'a mut Json, name: &str) -> Option<&'a mut Json> {
    let index = push_named(object, name, create_array())?;
    object_items_mut(object)?.get_mut(index)
}

pub fn detach_item_from_array(array: &mut Json, which: i32) -> Option<Json> {
    if which < 0 {
        return None;
    }
    let items = children_mut(array)?;
    let index = which as usize;
    if index >= items.len() {
        return None;
    }
    Some(items.remove(index))
}

pub fn delete_item_from_array(array: &mut Json, which: i32) {
    if let Some(item) = detach_item_from_array(array, which) {
        delete(item);
    }
}

pub fn detach_item_from_object(object: &mut Json, name: &str) -> Option<Json> {
    detach_item_from_object_inner(object, name, false)
}

pub fn detach_item_from_object_case_sensitive(object: &mut Json, name: &str) -> Option<Json> {
    detach_item_from_object_inner(object, name, true)
}

fn detach_item_from_object_inner(
    object: &mut Json,
    name: &str,
    case_sensitive: bool,
) -> Option<Json> {
    let items = match &mut object.value {
        Value::Object(items) => items,
        _ => return None,
    };
    let index = find_child(items, name, case_sensitive)?;
    Some(items.remove(index))
}

pub fn delete_item_from_object(object: &mut Json, name: &str) {
    if let Some(item) = detach_item_from_object(object, name) {
        delete(item);
    }
}

pub fn delete_item_from_object_case_sensitive(object: &mut Json, name: &str) {
    if let Some(item) = detach_item_from_object_case_sensitive(object, name) {
        delete(item);
    }
}

pub fn insert_item_in_array(array: &mut Json, which: i32, item: Json) -> bool {
    if which < 0 {
        return false;
    }
    let Value::Array(items) = &mut array.value else {
        return false;
    };
    let index = which as usize;
    if index >= items.len() {
        items.push(item);
    } else {
        items.insert(index, item);
    }
    true
}

pub fn replace_item_in_array(array: &mut Json, which: i32, newitem: Json) -> bool {
    if which < 0 {
        return false;
    }
    let Value::Array(items) = &mut array.value else {
        return false;
    };
    let index = which as usize;
    if index >= items.len() {
        return false;
    }
    items[index] = newitem;
    true
}

pub fn replace_item_in_object(object: &mut Json, name: &str, newitem: Json) -> bool {
    replace_item_in_object_inner(object, name, newitem, false)
}

pub fn replace_item_in_object_case_sensitive(object: &mut Json, name: &str, newitem: Json) -> bool {
    replace_item_in_object_inner(object, name, newitem, true)
}

fn replace_item_in_object_inner(
    object: &mut Json,
    name: &str,
    mut newitem: Json,
    case_sensitive: bool,
) -> bool {
    newitem.key = Some(Key {
        name: name.to_owned(),
        constant: false,
    });
    let Value::Object(items) = &mut object.value else {
        return false;
    };
    let Some(index) = find_child(items, name, case_sensitive) else {
        return false;
    };
    items[index] = newitem;
    true
}

pub fn duplicate(item: &Json, recurse: bool) -> Option<Json> {
    duplicate_rec(item, 0, recurse)
}

fn duplicate_rec(item: &Json, depth: usize, recurse: bool) -> Option<Json> {
    if depth >= CIRCULAR_LIMIT {
        return None;
    }
    let value = match &item.value {
        Value::Array(items) if recurse => Value::Array(duplicate_list(items, depth)?),
        Value::Object(items) if recurse => Value::Object(duplicate_list(items, depth)?),
        Value::Array(_) => Value::Array(Vec::new()),
        Value::Object(_) => Value::Object(Vec::new()),
        Value::Bool(value) => Value::Bool(*value),
        Value::Null => Value::Null,
        Value::Number {
            valuedouble,
            valueint,
        } => Value::Number {
            valuedouble: *valuedouble,
            valueint: *valueint,
        },
        Value::String(text) => Value::String(text.clone()),
        Value::Raw(text) => Value::Raw(text.clone()),
    };
    Some(Json {
        value,
        key: item.key.clone(),
        reference: false,
    })
}

fn duplicate_list(items: &[Json], depth: usize) -> Option<Vec<Json>> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(duplicate_rec(item, depth + 1, true)?);
    }
    Some(out)
}

pub fn compare(a: &Json, b: &Json, case_sensitive: bool) -> bool {
    if std::ptr::eq(a, b) {
        return true;
    }
    if std::mem::discriminant(&a.value) != std::mem::discriminant(&b.value) {
        return false;
    }
    match (&a.value, &b.value) {
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::Null, Value::Null) => true,
        (
            Value::Number {
                valuedouble: left, ..
            },
            Value::Number {
                valuedouble: right, ..
            },
        ) => compare_double(*left, *right),
        (Value::String(left), Value::String(right)) | (Value::Raw(left), Value::Raw(right)) => {
            c_bytes(left) == c_bytes(right)
        }
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(a_item, b_item)| compare(a_item, b_item, case_sensitive))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.iter()
                .all(|a_item| object_pair_eq(a_item, right, case_sensitive))
                && right
                    .iter()
                    .all(|b_item| object_pair_eq(b_item, left, case_sensitive))
        }
        _ => false,
    }
}

fn object_pair_eq(item: &Json, others: &[Json], case_sensitive: bool) -> bool {
    let Some(name) = item.key.as_ref() else {
        return false;
    };
    let Some(index) = find_child(others, &name.name, case_sensitive) else {
        return false;
    };
    compare(item, &others[index], case_sensitive)
}

pub fn set_number_helper(object: &mut Json, number: f64) -> f64 {
    if let Value::Number {
        valuedouble,
        valueint,
    } = &mut object.value
    {
        *valueint = value_int(number);
        *valuedouble = number;
        number
    } else {
        f64::NAN
    }
}

pub fn set_int_value(object: &mut Json, number: i32) -> i32 {
    if let Value::Number {
        valuedouble,
        valueint,
    } = &mut object.value
    {
        *valueint = number;
        *valuedouble = f64::from(number);
    }
    number
}

pub fn set_number_value(object: &mut Json, number: f64) -> f64 {
    set_number_helper(object, number)
}

pub fn set_valuestring<'a>(object: &'a mut Json, valuestring: &str) -> Option<&'a str> {
    if object.reference {
        return None;
    }
    match &mut object.value {
        Value::String(text) => {
            *text = valuestring.to_owned();
            Some(c_prefix(text))
        }
        _ => None,
    }
}

pub fn set_bool_value(object: &mut Json, bool_value: bool) -> i32 {
    if !matches!(object.value, Value::Bool(_)) {
        return TYPE_INVALID;
    }
    object.value = Value::Bool(bool_value);
    if bool_value {
        TYPE_TRUE
    } else {
        TYPE_FALSE
    }
}
