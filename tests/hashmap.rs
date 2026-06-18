#[test]
fn hashmap_string_values() {
    use std::collections::HashMap;
    let m: HashMap<String, String> =
        serde_kv::from_str(r#"foo=bar baz=42 msg="hello world""#).unwrap();
    assert_eq!(m["foo"], "bar");
    assert_eq!(m["baz"], "42");
    assert_eq!(m["msg"], "hello world");
}

#[test]
fn hashmap_typed_values() {
    use std::collections::HashMap;
    let m: HashMap<String, u64> = serde_kv::from_str("a=1 b=2").unwrap();
    assert_eq!(m["a"], 1);
    assert_eq!(m["b"], 2);
}

#[test]
fn hashmap_roundtrip() {
    use std::collections::BTreeMap;
    let mut m = BTreeMap::new();
    m.insert("a".to_string(), "x y".to_string());
    m.insert("b".to_string(), "z".to_string());
    let s = serde_kv::to_string(&m).unwrap();
    assert_eq!(s, r#"a="x y" b=z"#);
    let back: BTreeMap<String, String> = serde_kv::from_str(&s).unwrap();
    assert_eq!(back, m);
}
