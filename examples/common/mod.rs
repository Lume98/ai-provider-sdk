use std::{collections::HashMap, fs, path::Path};

use vendor_ai_sdk::{OpenAIClient, OpenAIConfig};

const EXAMPLE_CONFIG_PATH: &str = "openai.env";
const API_KEY_KEY: &str = "OPENAI_API_KEY";
const BASE_URL_KEY: &str = "OPENAI_BASE_URL";

pub fn client() -> OpenAIClient {
    vendor_ai_sdk::init_default_logger();

    let config_map = load_env_file(EXAMPLE_CONFIG_PATH).unwrap_or_else(|err| {
        panic!(
            "读取 examples 配置失败: {err}\n请创建 {}（可参考 openai.env.example）",
            EXAMPLE_CONFIG_PATH
        )
    });

    let api_key = config_map
        .get(API_KEY_KEY)
        .cloned()
        .or_else(|| std::env::var(API_KEY_KEY).ok())
        .unwrap_or_else(|| {
            panic!(
                "缺少 {API_KEY_KEY}，请在 {} 或环境变量中配置",
                EXAMPLE_CONFIG_PATH
            )
        });

    let mut config = OpenAIConfig::new(api_key);
    if let Some(base_url) = config_map
        .get(BASE_URL_KEY)
        .cloned()
        .or_else(|| std::env::var(BASE_URL_KEY).ok())
    {
        config = config.with_base_url(base_url);
    }

    OpenAIClient::from_config(config)
}

fn load_env_file(path: &str) -> Result<HashMap<String, String>, String> {
    let path = Path::new(path);
    let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let mut map = HashMap::new();

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (key, value) = trimmed
            .split_once('=')
            .ok_or_else(|| format!("第 {} 行格式错误，需为 KEY=VALUE", idx + 1))?;
        map.insert(key.trim().to_string(), value.trim().to_string());
    }

    Ok(map)
}
