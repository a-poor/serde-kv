use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct WithExtra {
    foo: String,
    baz: u64,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

#[test]
fn flatten_collects_extra_string_keys() {
    let got: WithExtra = serde_kv::from_str(r#"foo=bar baz=42 a=hi b="x y""#).unwrap();
    assert_eq!(got.foo, "bar");
    assert_eq!(got.baz, 42);
    assert_eq!(got.extra.get("a").map(String::as_str), Some("hi"));
    assert_eq!(got.extra.get("b").map(String::as_str), Some("x y"));
    assert_eq!(got.extra.len(), 2);
}

#[test]
fn flatten_serialize() {
    let mut extra = std::collections::BTreeMap::new();
    extra.insert("a".to_string(), "hi".to_string());
    #[derive(Serialize)]
    struct W {
        foo: String,
        #[serde(flatten)]
        extra: std::collections::BTreeMap<String, String>,
    }
    let s = serde_kv::to_string(&W { foo: "bar".into(), extra }).unwrap();
    assert_eq!(s, "foo=bar a=hi");
}
