# Moderations

## 如何使用

```rust
use ai_provider_sdk::{ModerationCreateParams, OpenAI};

# async fn demo() -> Result<(), ai_provider_sdk::Error> {
let client = OpenAI::from_env()?;

let resp = client
    .moderations()
    .create(ModerationCreateParams::new("hello").model("omni-moderation-latest"))
    .await?;

println!("{}", resp.id);
# Ok(())
# }
```

## 已实现方法

- `create(params)`
- `create_with_options(params, options)`

## 入参结构（全量）

`ModerationCreateParams`

- `input: ModerationInput`（必填）待审核输入。
- `model: Option<String>`（可选）审核模型 ID。
- `extra: HashMap<String, Value>`（可选）扩展字段。

`ModerationInput`（联合类型）

- `Text(String)`：单条文本。
- `Texts(Vec<String>)`：批量文本。
- `Items(Vec<ModerationInputItem>)`：多模态输入项。

`ModerationInputItem`（tagged enum）

- `Text { text: String }`
- `ImageUrl { image_url: ModerationImageUrl }`

`ModerationImageUrl`

- `url: String`

便捷构造：

- `ModerationCreateParams::new(input)`
- `.model(model)`
- `ModerationInputItem::text(text)`
- `ModerationInputItem::image_url(url)`

## 响应结构（全量）

`CreateModerationResponse`

- `id: String`
- `model: String`
- `results: Vec<ModerationResult>`
- `extra: HashMap<String, Value>`

`ModerationResult`

- `flagged: bool`
- `categories: ModerationCategories`
- `category_scores: ModerationCategoryScores`
- `category_applied_input_types: Option<ModerationCategoryAppliedInputTypes>`
- `extra: HashMap<String, Value>`

`ModerationCategories`（所有字段均为 `Option<bool>`）

- `sexual`
- `hate`
- `harassment`
- `self_harm`（wire: `self-harm`）
- `sexual_minors`（wire: `sexual/minors`）
- `hate_threatening`（wire: `hate/threatening`）
- `violence_graphic`（wire: `violence/graphic`）
- `self_harm_intent`（wire: `self-harm/intent`）
- `self_harm_instructions`（wire: `self-harm/instructions`）
- `harassment_threatening`（wire: `harassment/threatening`）
- `violence`
- `illicit`
- `illicit_violent`（wire: `illicit/violent`）
- `extra: HashMap<String, Value>`

`ModerationCategoryScores`（所有字段均为 `Option<f64>`，字段集合同上）

- `sexual` / `hate` / `harassment` / `self_harm` / `sexual_minors`
- `hate_threatening` / `violence_graphic` / `self_harm_intent` / `self_harm_instructions`
- `harassment_threatening` / `violence` / `illicit` / `illicit_violent`
- `extra: HashMap<String, Value>`

`ModerationCategoryAppliedInputTypes`（所有字段均为 `Option<Vec<ModerationAppliedInputType>>`，字段集合同上）

- `sexual` / `hate` / `harassment` / `self_harm` / `sexual_minors`
- `hate_threatening` / `violence_graphic` / `self_harm_intent` / `self_harm_instructions`
- `harassment_threatening` / `violence` / `illicit` / `illicit_violent`
- `extra: HashMap<String, Value>`

`ModerationAppliedInputType` 枚举值：

- `Text`
- `Image`

## 兼容性说明

- `extra` 为前向兼容容器，不保证稳定结构。
- 文档只覆盖当前仓库已实现能力。
