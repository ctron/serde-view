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
//!     id: String,
//!     #[serde(default)]
//!     name: String,
//!     #[serde(default)]
//!     tags: Vec<String>,
//! }
//!
//! fn serialize(my: &MyStruct) -> Result<serde_json::Value, serde_json::Error> {
//!     serde_json::to_value(my.as_view().with_fields([
//!         <MyStruct as View>::Fields::Id,
//!         <MyStruct as View>::Fields::Name,
//!     ]).unwrap())
//! }
//!
//! fn serialize_str_fields(my: &MyStruct) -> Result<serde_json::Value, serde_json::Error> {
//!     // as fields can be converted to strings, it is also possible to pass something like a
//!     // comma separated list
//!     serde_json::to_value(my.as_view().with_fields(
//!         <MyStruct as View>::Fields::from_str_split("id,name").unwrap()
//!     ).unwrap())
//! }
//! ```

mod ser;

pub use ser::*;
pub use serde_view_macros::View;

use std::{collections::HashSet, fmt, hash::Hash};

pub trait ViewFields: Clone + Copy + Hash + PartialEq + Eq {
    fn as_str(&self) -> &'static str;
    fn from_str(name: &str) -> Result<Self>;

    fn from_str_iter<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<HashSet<Self>> {
        names.into_iter().map(Self::from_str).collect()
    }

    fn from_str_split(names: &str) -> Result<HashSet<Self>> {
        Self::from_str_iter(names.split(','))
    }
}

#[derive(Debug, serde::Serialize)]
pub enum Error {
    UnknownField(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

pub trait IntoField<VF: ViewFields> {
    fn into_field(self) -> Result<VF>;
}

/// Implement for all fields.
impl<VF: ViewFields> IntoField<VF> for VF {
    fn into_field(self) -> Result<VF> {
        Ok(self)
    }
}

/// Implement for `&str`, using [`ViewFields::from_str`].
impl<VF: ViewFields> IntoField<VF> for &str {
    fn into_field(self) -> Result<VF> {
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
    pub fn with_fields<I, IF>(mut self, fields: I) -> Result<Self>
    where
        I: IntoIterator<Item = IF>,
        IF: IntoField<T::Fields>,
    {
        self.fields = fields
            .into_iter()
            .map(|f| f.into_field())
            .collect::<Result<_>>()?;
        Ok(self)
    }

    pub fn add_fields<I, IF>(mut self, fields: I) -> Result<Self>
    where
        I: IntoIterator<Item = IF>,
        IF: IntoField<T::Fields>,
    {
        self.fields.extend(
            fields
                .into_iter()
                .map(|f| f.into_field())
                .collect::<Result<HashSet<T::Fields>>>()?,
        );
        Ok(self)
    }

    pub fn add_field(mut self, field: impl IntoField<T::Fields>) -> Result<Self> {
        self.fields.insert(field.into_field()?);
        Ok(self)
    }
}

pub trait View: Sized + serde::Serialize {
    type Fields: ViewFields;

    fn as_view(&self) -> ViewContext<Self> {
        ViewContext {
            inner: self,
            fields: Default::default(),
        }
    }
}
