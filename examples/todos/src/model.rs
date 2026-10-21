use std::sync::Arc;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, VariantNames};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub done: bool,
    pub priority: Priority,
}

#[derive(
    Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter, VariantNames,
)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

pub type Todos = Arc<Mutex<Vec<Todo>>>;

#[derive(
    Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter, VariantNames,
)]
pub enum TodoFilter {
    #[default]
    All,
    Active,
    Done,
}

impl TodoFilter {
    pub fn matches(self, todo: &Todo) -> bool {
        match self {
            TodoFilter::All => true,
            TodoFilter::Active => !todo.done,
            TodoFilter::Done => todo.done,
        }
    }
}
