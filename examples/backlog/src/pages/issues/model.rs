use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, FromRepr};

pub struct Issue {
    pub id: i64,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub estimate: Option<i64>,
    pub due: Option<String>,
    pub open: bool,
    pub overdue: bool,
}

#[derive(Default)]
pub struct Stats {
    pub open: i64,
    pub in_progress: i64,
    pub points: i64,
    pub overdue: i64,
}

/// One edited cell.
pub enum Change {
    Title(String),
    Status(Status),
    Priority(Priority),
    Estimate(Option<i64>),
    Due(Option<String>),
}

#[derive(
    Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter, FromRepr, Debug,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum Status {
    Backlog,
    #[default]
    Todo,
    InProgress,
    Done,
    Canceled,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Backlog => "Backlog",
            Status::Todo => "Todo",
            Status::InProgress => "In progress",
            Status::Done => "Done",
            Status::Canceled => "Canceled",
        }
    }

    pub fn tone(self) -> &'static str {
        match self {
            Status::Backlog | Status::Canceled => "bg-muted-foreground/40",
            Status::Todo => "bg-foreground",
            Status::InProgress => "bg-amber-500",
            Status::Done => "bg-emerald-500",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter, FromRepr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Priority::None => "—",
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Urgent => "Urgent",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, Debug)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SortField {
    #[default]
    Created,
    Title,
    Status,
    Priority,
    Estimate,
    Due,
}

// Stored by discriminant, so the columns sort in declaration order.
macro_rules! stored_as_integer {
    ($($ty:ty),*) => {$(
        impl ToSql for $ty {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                Ok(ToSqlOutput::from(*self as u8))
            }
        }

        impl FromSql for $ty {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let repr = value.as_i64()?;

                u8::try_from(repr)
                    .ok()
                    .and_then(Self::from_repr)
                    .ok_or(FromSqlError::OutOfRange(repr))
            }
        }
    )*};
}

stored_as_integer!(Status, Priority);
