//! 游标分页抽象。
//!
//! 提供 OpenAI 风格游标分页的通用容器 [`CursorPage<T>`]，
//! 用于 `list` 接口在不同资源间复用翻页语义。
//!
//! ## 分页协议
//!
//! OpenAI list API 通常返回如下结构：
//! ```json
//! {
//!   "object": "list",
//!   "data": [...],
//!   "has_more": true,
//!   ...未知字段进入 extra
//! }
//! ```
//!
//! 翻页方式：将当前页最后一个元素的 `id` 作为下一页请求的 `after` 参数。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 可参与游标翻页的元素约束。
///
/// 约定：实现方应返回服务端稳定 ID，用于构建下一页请求的 `after` 游标。
/// 典型实现：`fn id(&self) -> Option<&str>` 直接返回 `self.id`。
pub trait CursorPageItem {
    /// 返回元素的服务端 ID；用于构建下一页的游标参数。
    fn id(&self) -> Option<&str>;
}

/// OpenAI 风格游标分页容器。
///
/// 泛型参数 `T` 为列表元素类型，需实现 [`CursorPageItem`] 以支持 `has_next_page()` 等方法。
/// `extra` 字段使用 `#[serde(flatten)]` 捕获未知字段，避免服务端增量字段导致反序列化失败。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CursorPage<T> {
    /// 分页对象类型标识（通常为 `"list"`）。
    #[serde(default)]
    pub object: Option<String>,
    /// 当前页的数据元素列表。
    pub data: Vec<T>,
    /// 服务端指示是否还有更多数据。
    ///
    /// - `Some(true)`：明确有下一页。
    /// - `Some(false)`：明确没有下一页。
    /// - `None`：未返回该字段，由客户端根据 `data` 是否为空推断。
    #[serde(default)]
    pub has_more: Option<bool>,
    /// 未知字段的前向兼容保留。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl<T> CursorPage<T> {
    /// 获取当前页元素的切片引用。
    pub fn items(&self) -> &[T] {
        &self.data
    }

    /// 消费当前页，返回元素列表。
    pub fn into_items(self) -> Vec<T> {
        self.data
    }
}

impl<T: CursorPageItem> CursorPage<T> {
    /// 判断是否存在下一页。
    ///
    /// 判定逻辑：
    /// 1. 如果 `has_more` 明确为 `false`，返回 `false`。
    /// 2. 否则检查最后一个元素是否提供了 `id`（即 `next_after()` 是否为 `Some`）。
    pub fn has_next_page(&self) -> bool {
        if self.has_more == Some(false) {
            return false;
        }

        self.next_after().is_some()
    }

    /// 返回下一页的游标值（即当前页最后一个元素的 `id`）。
    ///
    /// 当 `data` 为空或最后一个元素无 `id` 时返回 `None`。
    pub fn next_after(&self) -> Option<&str> {
        self.data.last().and_then(CursorPageItem::id)
    }
}
