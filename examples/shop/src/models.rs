use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Category {
    Kitchen,
    Textiles,
    Tableware,
    Woodwork,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Kitchen => "Kitchen",
            Category::Textiles => "Textiles",
            Category::Tableware => "Tableware",
            Category::Woodwork => "Woodwork",
        }
    }
}

impl FromSql for Category {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|_| FromSqlError::InvalidType)
    }
}

impl ToSql for Category {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_ref()))
    }
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ProductSort {
    #[default]
    Shelf,
    PriceAsc,
    PriceDesc,
    Name,
}

impl ProductSort {
    pub fn label(self) -> &'static str {
        match self {
            ProductSort::Shelf => "As shelved",
            ProductSort::PriceAsc => "Price, low to high",
            ProductSort::PriceDesc => "Price, high to low",
            ProductSort::Name => "Name",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum MaterialSort {
    #[default]
    Name,
    MostUsed,
    LeastUsed,
}

impl MaterialSort {
    pub fn label(self) -> &'static str {
        match self {
            MaterialSort::Name => "Name",
            MaterialSort::MostUsed => "Most used",
            MaterialSort::LeastUsed => "Least used",
        }
    }
}

/// The last row of a page: its id and its sort key, `{id}:{key}` on the wire.
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct After {
    pub id: i64,
    pub key: String,
}

impl From<After> for String {
    fn from(after: After) -> Self {
        format!("{}:{}", after.id, after.key)
    }
}

impl TryFrom<String> for After {
    type Error = &'static str;

    fn try_from(cursor: String) -> Result<Self, Self::Error> {
        let (id, key) = cursor.split_once(':').ok_or("not a cursor")?;

        Ok(After {
            id: id.parse().map_err(|_| "not a cursor")?,
            key: key.to_owned(),
        })
    }
}

/// What a multiple combobox posts: basecoat's object format, a JSON array in one field.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Selection(pub Vec<Picked>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Picked {
    pub value: String,
    pub label: String,
}

impl From<Selection> for String {
    fn from(selection: Selection) -> Self {
        serde_json::to_string(&selection.0).expect("a selection serializes")
    }
}

impl TryFrom<String> for Selection {
    type Error = serde_json::Error;

    fn try_from(json: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&json).map(Selection)
    }
}
