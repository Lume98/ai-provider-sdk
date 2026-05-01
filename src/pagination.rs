use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub trait CursorPageItem {
    fn id(&self) -> Option<&str>;
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CursorPage<T> {
    #[serde(default)]
    pub object: Option<String>,
    pub data: Vec<T>,
    #[serde(default)]
    pub has_more: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl<T> CursorPage<T> {
    pub fn items(&self) -> &[T] {
        &self.data
    }

    pub fn into_items(self) -> Vec<T> {
        self.data
    }
}

impl<T: CursorPageItem> CursorPage<T> {
    pub fn has_next_page(&self) -> bool {
        if self.has_more == Some(false) {
            return false;
        }

        self.next_after().is_some()
    }

    pub fn next_after(&self) -> Option<&str> {
        self.data.last().and_then(CursorPageItem::id)
    }
}
