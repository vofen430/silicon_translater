use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};

const DEFAULT_ENDPOINT: &str = "https://api.siliconflow.cn/v1/chat/completions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated_text: String,
    pub detected_source_lang: Option<String>,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("missing api key")]
    MissingApiKey,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("rate limited")]
    RateLimited,
    #[error("model unavailable")]
    ModelUnavailable,
    #[error("unexpected response: {0}")]
    Unexpected(String),
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    endpoint: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("http client");
        Self {
            endpoint: DEFAULT_ENDPOINT.to_string(),
            http,
        }
    }

    pub async fn translate(
        &self,
        request: TranslationRequest,
        api_key: Option<String>,
    ) -> Result<TranslationResponse, ApiError> {
        let api_key = api_key.ok_or(ApiError::MissingApiKey)?;
        let payload = ChatCompletionRequest::from(request);
        let mut attempt = 0;
        let max_attempts = 3;

        loop {
            attempt += 1;
            let response = self
                .http
                .post(&self.endpoint)
                .bearer_auth(&api_key)
                .json(&payload)
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let body: ChatCompletionResponse = response.json().await?;
                    let translated = body
                        .choices
                        .first()
                        .and_then(|choice| choice.message.content.clone())
                        .ok_or_else(|| ApiError::Unexpected("empty response".into()))?;

                    return Ok(TranslationResponse {
                        translated_text: translated,
                        detected_source_lang: None,
                    });
                }
                StatusCode::UNAUTHORIZED => return Err(ApiError::Unauthorized),
                StatusCode::TOO_MANY_REQUESTS => return Err(ApiError::RateLimited),
                StatusCode::SERVICE_UNAVAILABLE => return Err(ApiError::ModelUnavailable),
                status if status.is_server_error() && attempt < max_attempts => {
                    let backoff = 200_u64 * 2_u64.pow(attempt - 1);
                    warn!(status = ?status, attempt, "server error, retrying");
                    tokio::time::sleep(Duration::from_millis(backoff)).await;
                }
                status => {
                    let text = response.text().await.unwrap_or_default();
                    return Err(ApiError::Unexpected(format!(
                        "status {status}, body {text}"
                    )));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    top_p: f32,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatMessageContent {
    content: Option<String>,
}

impl From<TranslationRequest> for ChatCompletionRequest {
    fn from(request: TranslationRequest) -> Self {
        let system_prompt = "你是专业翻译。忠实准确，保持术语一致，不扩写不发挥。只输出译文。";
        let user_prompt = format!(
            "将以下文本从 {} 翻译为 {}:\n{}",
            request.source_lang, request.target_lang, request.text
        );

        info!(model = %request.model, "translation request");

        Self {
            model: request.model,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: 0.2,
            top_p: 0.95,
            stream: false,
        }
    }
}
