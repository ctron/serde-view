//! Serialize a view of a structure
//!
//! The idea of this crate is to serialize only a sub-set of fields of a struct, making the
//! decision which fields during runtime.
//!
//! ## Basic example
//!
//! Assume you have a struct like:
//!
//! ```rust
//! #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
//! pub struct MyStruct {
//!     id: String,
//!     #[serde(default)]
//!     name: String,
//!     #[serde(default)]
//!     tags: Vec<String>,
//! }
//! ```
//!
//! Now, you want to make it possible to only serialize a sub-set of the data, making this
//! decision during runtime. This can be done by adding the [`View`] derive to the struct, and
//! wrapping the serializing with the [`View::view`] function, adding extra information to the
//! view context:
//!
//! ```rust
//! use serde_view::View;
//! use serde_view::ViewFields;
//!
//! #[derive(Clone, Debug, serde::Deserialize, serde::Serialize, View)]
//! pub struct MyStruct {
//! #    id: String,
//! #    #[serde(default)]
//! #    name: String,
//! #    #[serde(default)]
//! #    tags: Vec<String>,
//! }
//!
//! fn serialize(my: &MyStruct) -> Result<serde_json::Value, serde_json::Error> {
//!     serde_json::to_value(my.as_view().with_fields([
//!         <MyStruct as View>::Fields::Id,
//!         <MyStruct as View>::Fields::Name,
//!     ]))
//! }
//!
//! fn serialize_str_fields(my: &MyStruct) -> Result<serde_json::Value, serde_json::Error> {
//!     // as fields can be converted to strings, it is also possible to something like a
//!     // comma separated list
//!     serde_json::to_value(my.as_view().with_fields(
//!         <MyStruct as View>::Fields::from_str_iter("id,name".split(","))
//!     ))
//! }
//! ```

mod ser;

pub use ser::*;
pub use serde_view_macros::View;

use std::{collections::HashSet, hash::Hash};

pub trait ViewFields: Clone + Copy + Hash + PartialEq + Eq {
    fn as_str(&self) -> &'static str;
    fn from_str(name: &str) -> Option<Self>;

    fn from_str_iter<'a>(names: impl IntoIterator<Item = &'a str>) -> HashSet<Self> {
        names.into_iter().filter_map(Self::from_str).collect()
    }

    fn from_str_split(names: &str) -> HashSet<Self> {
        Self::from_str_iter(names.split(","))
    }
}

/// Convert something into a field, with the option to silently fail.
pub trait IntoField<VF: ViewFields> {
    fn into_field(self) -> Option<VF>;
}

/// Implement for all fields.
impl<VF: ViewFields> IntoField<VF> for VF {
    fn into_field(self) -> Option<VF> {
        Some(self)
    }
}

/// Implement for `&str`, using [`ViewFields::from_str`].
impl<VF: ViewFields> IntoField<VF> for &str {
    fn into_field(self) -> Option<VF> {
        VF::from_str(self)
    }
}

pub struct ViewContext<'v, T>
where
    T: View,
{
    inner: &'v T,
    fields: HashSet<T::Fields>,
}

impl<'v, T> ViewContext<'v, T>
where
    T: View,
{
    pub fn with_fields<I, IF>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = IF>,
        IF: IntoField<T::Fields>,
    {
        self.fields = fields.into_iter().filter_map(|f| f.into_field()).collect();
        self
    }

    pub fn add_fields<I, IF>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = IF>,
        IF: IntoField<T::Fields>,
    {
        self.fields
            .extend(fields.into_iter().filter_map(|f| f.into_field()));
        self
    }

    pub fn add_field(mut self, field: impl IntoField<T::Fields>) -> Self {
        if let Some(field) = field.into_field() {
            self.fields.insert(field);
        }
        self
    }
}

pub trait View: Sized + serde::Serialize {
    type Fields: ViewFields;

    fn as_view(self: &Self) -> ViewContext<Self> {
        ViewContext {
            inner: self,
            fields: Default::default(),
        }
    }
}
