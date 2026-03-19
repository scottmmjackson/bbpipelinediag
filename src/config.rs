use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub bitbucket_username: Option<String>,
    pub bitbucket_app_password: Option<String>,
    pub http_range: Option<HttpRangeConfig>,
    #[serde(default)]
    pub filters: Vec<FilterConfig>,
    pub diagnosis: Option<DiagnosisConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpRangeConfig {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FilterConfig {
    Head {
        n: usize,
    },
    Tail {
        n: usize,
    },
    Grep {
        pattern: String,
        #[serde(default)]
        before: usize,
        #[serde(default)]
        after: usize,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DiagnosisConfig {
    Llm {
        #[serde(flatten)]
        config: LlmConfig,
    },
    Exec {
        command: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LlmConfig {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt: Option<String>,
}
