use serde_json::json;
use serde_view::{View, ViewFields};
use std::fmt::Formatter;

#[derive(Clone, Debug, serde_view::View, serde::Serialize, serde::Deserialize)]
pub struct MyRecordDerived {
    some_string: String,
    flag: bool,
    optional_flag: Option<bool>,
}

impl Default for MyRecordDerived {
    fn default() -> Self {
        Self {
            some_string: "Hello World".to_string(),
            flag: true,
            optional_flag: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MyRecord {
    some_string: String,
    flag: bool,
    optional_flag: Option<bool>,
}

impl Default for MyRecord {
    fn default() -> Self {
        Self {
            some_string: "Hello World".to_string(),
            flag: true,
            optional_flag: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MyRecordFields {
    SomeString,
    Flag,
    OptionalFlag,
}

impl ViewFields for MyRecordFields {
    fn as_str(&self) -> &'static str {
        match self {
            Self::SomeString => "some_string",
            Self::Flag => "flag",
            Self::OptionalFlag => "optional_flag",
        }
    }

    fn from_str(name: &str) -> Option<Self> {
        Some(match name {
            "some_string" => Self::SomeString,
            "flag" => Self::Flag,
            "optional_flag" => Self::OptionalFlag,
            _ => return None,
        })
    }
}

impl std::fmt::Display for MyRecordFields {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl View for MyRecord {
    type Fields = MyRecordFields;
}

#[test]
fn test_manual() {
    assert_eq!(
        serde_json::to_value(
            MyRecord::default()
                .as_view()
                .with_fields([<MyRecord as View>::Fields::SomeString])
        )
        .unwrap(),
        json!({
            "some_string": "Hello World",
        })
    )
}

#[test]
fn test_derived() {
    assert_eq!(
        serde_json::to_value(
            MyRecordDerived::default()
                .as_view()
                .with_fields([<MyRecordDerived as View>::Fields::SomeString])
        )
        .unwrap(),
        json!({
            "some_string": "Hello World",
        })
    );

    assert_eq!(
        serde_json::to_value(
            MyRecordDerived::default()
                .as_view()
                .with_fields([MyRecordDerivedFields::SomeString])
        )
        .unwrap(),
        json!({
            "some_string": "Hello World",
        })
    );

    assert_eq!(
        serde_json::to_value(MyRecordDerived::default().as_view().with_fields(
            <MyRecordDerived as View>::Fields::from_str_iter("some_string,flag".split(","))
        ))
        .unwrap(),
        json!({
            "flag": true,
            "some_string": "Hello World",
        })
    );
}

#[test]
fn test_all() {
    // if no fields are selected, this means: all
    let mut value = MyRecordDerived::default();
    value.optional_flag = Some(false);

    assert_eq!(
        serde_json::to_value(value.as_view()).unwrap(),
        json!({
            "flag": true,
            "some_string": "Hello World",
            "optional_flag": false,
        })
    );
}
