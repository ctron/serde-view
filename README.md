# Serialize views of data

Dynamically select during serialization which fields will be included.

## Example

```rust
use serde_view::View;
use serde_view::ViewFields;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, View)]
pub struct MyStruct {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn serialize(my: &MyStruct) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(my.as_view().with_fields([
       <MyStruct as View>::Fields::Id,
       <MyStruct as View>::Fields::Name,
   ]).unwrap())
}
```
