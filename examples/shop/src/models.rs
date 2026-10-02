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
