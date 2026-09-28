use cjson::{
    add_item_reference_to_array, add_item_to_object_cs, add_number_to_object, add_string_to_object,
    compare, create_array, create_false, create_null, create_number, create_object, create_string,
    create_true, delete, delete_item_from_array, detach_item_from_object, duplicate,
    get_array_item, get_array_item_mut, get_array_size, get_error_ptr, get_number_value,
    get_object_item, get_object_item_case_sensitive, get_string_value, has_object_item,
    insert_item_in_array, is_array, is_bool, is_false, is_null, is_number, is_object, is_reference,
    is_string, is_true, key, minify, parse, parse_with_opts, print, print_buffered,
    print_preallocated, print_unformatted, replace_item_in_array, set_bool_value, set_number_value,
    set_valuestring, string_is_const, type_bits, version, TYPE_FALSE, TYPE_INVALID, TYPE_NUMBER,
};

#[test]
fn version_is_the_vendored_release() {
    assert_eq!(version(), "1.7.19");
}

#[test]
fn parse_prints_scalars() {
    assert_eq!(
        print_unformatted(&parse("null").unwrap()).as_deref(),
        Some("null")
    );
    assert_eq!(
        print_unformatted(&parse("true").unwrap()).as_deref(),
        Some("true")
    );
    assert_eq!(
        print_unformatted(&parse("false").unwrap()).as_deref(),
        Some("false")
    );
    assert_eq!(
        print_unformatted(&parse("0").unwrap()).as_deref(),
        Some("0")
    );
    assert_eq!(
        print_unformatted(&parse("-2").unwrap()).as_deref(),
        Some("-2")
    );
    assert_eq!(
        print_unformatted(&parse("1.0").unwrap()).as_deref(),
        Some("1")
    );
    assert_eq!(
        print_unformatted(&parse("0.1").unwrap()).as_deref(),
        Some("0.1")
    );
}

#[test]
fn parse_prints_string_array_and_object() {
    let text = "  { \"a\" : [1, true, null, \"x\"] }  ";
    let value = parse(text).unwrap();
    assert_eq!(
        print_unformatted(&value).as_deref(),
        Some("{\"a\":[1,true,null,\"x\"]}")
    );
    assert!(is_object(&value));
    let array = get_object_item(&value, "A").unwrap();
    assert!(is_array(array));
    assert_eq!(get_array_size(array), 4);
    assert!(is_number(get_array_item(array, 0).unwrap()));
    assert!(is_true(get_array_item(array, 1).unwrap()));
    assert!(is_null(get_array_item(array, 2).unwrap()));
    assert_eq!(
        get_string_value(get_array_item(array, 3).unwrap()),
        Some("x")
    );
    assert!(get_object_item_case_sensitive(&value, "A").is_none());
    assert!(has_object_item(&value, "a"));
}

#[test]
fn escaped_string_round_trip() {
    let value = parse(r#""a\"b\\c\/d\n\t""#).unwrap();
    assert_eq!(get_string_value(&value), Some("a\"b\\c/d\n\t"));
    assert_eq!(
        print_unformatted(&value).as_deref(),
        Some(r#""a\"b\\c/d\n\t""#)
    );
}

#[test]
fn unicode_escape_prints_utf8() {
    let value = parse(r#""\u0041\u00e9""#).unwrap();
    assert_eq!(get_string_value(&value), Some("Aé"));
    assert_eq!(print_unformatted(&value).as_deref(), Some("\"Aé\""));
}

#[test]
fn reject_truncated_container_without_panic() {
    assert!(parse("[").is_none());
    assert!(get_error_ptr().is_some());
    assert!(parse("{").is_none());
    assert!(parse("\"unterminated").is_none());
    assert!(parse("").is_none());
}

#[test]
fn require_null_terminated_rejects_trailing_junk() {
    let ok = parse_with_opts("null", false);
    assert!(ok.item.is_some());
    assert!(is_null(ok.item.as_ref().unwrap()));
    let bad = parse_with_opts("null x", true);
    assert!(bad.item.is_none());
}

#[test]
fn builders_match_print() {
    let mut object = create_object();
    add_number_to_object(&mut object, "n", 1.0);
    add_string_to_object(&mut object, "s", "hi");
    assert_eq!(
        print_unformatted(&object).as_deref(),
        Some("{\"n\":1,\"s\":\"hi\"}")
    );
    assert_eq!(key(get_object_item(&object, "n").unwrap()), Some("n"));
    assert_eq!(
        type_bits(get_object_item(&object, "n").unwrap()) & TYPE_NUMBER,
        TYPE_NUMBER
    );
}

#[test]
fn array_insert_replace_and_delete() {
    let mut array = parse("[1,3]").unwrap();
    assert!(insert_item_in_array(&mut array, 1, create_number(2.0)));
    assert_eq!(print_unformatted(&array).as_deref(), Some("[1,2,3]"));
    assert!(replace_item_in_array(&mut array, 0, create_false()));
    assert!(is_false(get_array_item(&array, 0).unwrap()));
    delete_item_from_array(&mut array, 0);
    assert_eq!(print_unformatted(&array).as_deref(), Some("[2,3]"));
    delete(array);
}

#[test]
fn detach_object_item_keeps_the_rest() {
    let mut object = parse("{\"a\":1,\"b\":2}").unwrap();
    let detached = detach_item_from_object(&mut object, "A").unwrap();
    assert_eq!(get_number_value(&detached), 1.0);
    assert_eq!(print_unformatted(&object).as_deref(), Some("{\"b\":2}"));
}

#[test]
fn duplicate_and_compare() {
    let original = parse("{\"A\":[1,true]}").unwrap();
    let shallow = duplicate(&original, false).unwrap();
    assert_eq!(print_unformatted(&shallow).as_deref(), Some("{}"));
    let deep = duplicate(&original, true).unwrap();
    assert_eq!(
        print_unformatted(&deep).as_deref(),
        Some("{\"A\":[1,true]}")
    );
    let other_case = parse("{\"a\":[1,true]}").unwrap();
    assert!(compare(&deep, &other_case, false));
    assert!(!compare(&deep, &other_case, true));
    assert!(compare(&deep, &deep, true));
}

#[test]
fn set_helpers_change_printed_value() {
    let mut number = create_number(1.0);
    assert_eq!(set_number_value(&mut number, 2.5), 2.5);
    assert_eq!(print_unformatted(&number).as_deref(), Some("2.5"));
    let mut flag = create_true();
    assert!(is_bool(&flag) && is_true(&flag));
    assert_eq!(set_bool_value(&mut flag, false), TYPE_FALSE);
    assert!(is_false(&flag));
    let mut untouched = create_null();
    assert_eq!(set_bool_value(&mut untouched, true), TYPE_INVALID);
    assert!(is_null(&untouched));
    let mut text = create_string("ab");
    assert_eq!(set_valuestring(&mut text, "z"), Some("z"));
    assert_eq!(print_unformatted(&text).as_deref(), Some("\"z\""));
    assert!(set_valuestring(&mut number, "nope").is_none());
}

#[test]
fn formatted_object_uses_tabs() {
    let value = parse("{\"a\":1}").unwrap();
    assert_eq!(print(&value).as_deref(), Some("{\n\t\"a\":\t1\n}"));
}

#[test]
fn minify_drops_space_and_comments_and_keeps_strings() {
    let mut spaced = "{ \"a\" : 1 }".to_string();
    minify(&mut spaced);
    assert_eq!(spaced, "{\"a\":1}");

    let mut commented = "{ \"a\" : 1 } // hi\n/* x */".to_string();
    minify(&mut commented);
    assert_eq!(commented, "{\"a\":1}");

    let mut sentence = "{\"a\":\"b c\"}".to_string();
    minify(&mut sentence);
    assert_eq!(sentence, "{\"a\":\"b c\"}");

    let mut quoted = "{\"a\":\"say \\\"hi\\\"\"}".to_string();
    minify(&mut quoted);
    assert_eq!(quoted, "{\"a\":\"say \\\"hi\\\"\"}");

    let mut slash = "{\"a\":1}/".to_string();
    minify(&mut slash);
    assert_eq!(slash, "{\"a\":1}");
}

#[test]
fn print_buffer_helpers() {
    let value = parse("[1,2]").unwrap();
    assert!(print_buffered(&value, -1, false).is_none());
    assert_eq!(print_buffered(&value, 8, false).as_deref(), Some("[1,2]"));
    let mut buffer = [0u8; 8];
    assert!(print_preallocated(&value, &mut buffer, false));
    assert_eq!(&buffer[..6], b"[1,2]\0");
    let mut tiny = [0u8; 2];
    assert!(!print_preallocated(&value, &mut tiny, false));
}

#[test]
fn create_array_from_numbers() {
    let array = cjson::create_int_array(&[1, -2, 3]);
    assert_eq!(print_unformatted(&array).as_deref(), Some("[1,-2,3]"));
    let strings = cjson::create_string_array(&["a", "b"]);
    assert_eq!(
        print_unformatted(&strings).as_deref(),
        Some("[\"a\",\"b\"]")
    );
    assert!(is_string(get_array_item(&strings, 1).unwrap()));
}

#[test]
fn string_reference_refuses_set_and_still_prints() {
    let source = create_string("ref");
    let mut array = create_array();
    assert!(add_item_reference_to_array(&mut array, &source));
    assert_eq!(print_unformatted(&array).as_deref(), Some("[\"ref\"]"));
    let child = get_array_item_mut(&mut array, 0).unwrap();
    assert!(is_reference(child));
    assert!(set_valuestring(child, "x").is_none());
    assert_eq!(print_unformatted(&array).as_deref(), Some("[\"ref\"]"));
    assert_eq!(print_unformatted(&source).as_deref(), Some("\"ref\""));
}

#[test]
fn const_key_flag_is_visible() {
    let mut object = create_object();
    assert!(add_item_to_object_cs(&mut object, "k", create_true()));
    assert!(string_is_const(get_object_item(&object, "k").unwrap()));
    assert_eq!(print_unformatted(&object).as_deref(), Some("{\"k\":true}"));
}

#[test]
fn non_number_get_is_nan_and_missing_index_is_absent() {
    assert!(get_number_value(&create_array()).is_nan());
    assert!(get_string_value(&create_true()).is_none());
    assert!(get_array_item(&create_array(), -1).is_none());
    assert_eq!(get_array_size(&create_null()), 0);
}
