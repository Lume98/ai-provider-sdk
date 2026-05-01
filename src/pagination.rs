//! 游标分页抽象。用于 list 接口在不同资源间复用翻页语义。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 可参与游标翻页的元素约束。
///
/// 约定：实现方应返回服务端稳定 ID，用于构建下一页 `after`。
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
