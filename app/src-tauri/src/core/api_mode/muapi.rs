use crate::core::config::Config;
use crate::core::errors::ConfigError;
use crate::core::observability::events::{NoopProgressEmitter, ProgressEmitter};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum MuApiError {
    Config(ConfigError),
    Transport {
        stage: &'static str,
        message: String,
    },
    Api {
        stage: &'static str,
        message: String,
    },
}

impl std::fmt::Display for MuApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(err) => write!(f, "{err}"),
            Self::Transport { message, .. } | Self::Api { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for MuApiError {}

pub struct MuApiClient {
    config: Config,
    http: reqwest::Client,
    emitter: Arc<dyn ProgressEmitter>,
}

impl MuApiClient {
    pub fn new(config: Config) -> Self {
        Self::with_emitter(config, Arc::new(NoopProgressEmitter))
    }

    pub fn with_emitter(config: Config, emitter: Arc<dyn ProgressEmitter>) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            emitter,
        }
    }

    fn headers(&self) -> Result<HeaderMap, MuApiError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let key = self
            .config
            .require_api_key()
            .map_err(MuApiError::Config)?
            .to_string();
        let key_header = HeaderValue::from_str(&key).map_err(|err| MuApiError::Transport {
            stage: "submit",
            message: format!("invalid x-api-key header: {err}"),
        })?;
        headers.insert("x-api-key", key_header);
        Ok(headers)
    }

    pub async fn submit(
        &self,
        endpoint: &str,
        payload: &Value,
        retries: usize,
    ) -> Result<String, MuApiError> {
        let url = format!(
            "{}/{}",
            self.config.muapi_base_url,
            endpoint.trim_start_matches('/')
        );
        let mut last_err: Option<String> = None;

        for attempt in 0..retries {
            let result = self
                .http
                .post(&url)
                .headers(self.headers()?)
                .json(payload)
                .timeout(Duration::from_secs(120))
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_client_error() || status.is_server_error() {
                        let body = resp.text().await.unwrap_or_else(|_| "".to_string());
                        return Err(MuApiError::Api {
                            stage: "submit",
                            message: format!(
                                "{endpoint} submit failed [{}]: {body}",
                                status.as_u16()
                            ),
                        });
                    }
                    let data: Value = resp.json().await.map_err(|err| MuApiError::Transport {
                        stage: "submit",
                        message: format!("submit decode failed: {err}"),
                    })?;
                    if let Some(id) = data
                        .get("request_id")
                        .or_else(|| data.get("id"))
                        .and_then(|v| v.as_str())
                    {
                        return Ok(id.to_string());
                    }
                    return Err(MuApiError::Api {
                        stage: "submit",
                        message: format!("{endpoint} response had no request_id: {data}"),
                    });
                }
                Err(err) => {
                    if err.is_timeout() || err.is_connect() {
                        last_err = Some(err.to_string());
                        if attempt + 1 < retries {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    } else {
                        return Err(MuApiError::Transport {
                            stage: "submit",
                            message: err.to_string(),
                        });
                    }
                }
            }
        }

        Err(MuApiError::Api {
            stage: "submit",
            message: format!(
                "{endpoint} submit failed after {retries} retries: {}",
                last_err.unwrap_or_else(|| "unknown transport error".to_string())
            ),
        })
    }

    pub async fn fetch_result(
        &self,
        request_id: &str,
        retries: usize,
    ) -> Result<Value, MuApiError> {
        let url = format!(
            "{}/predictions/{request_id}/result",
            self.config.muapi_base_url
        );
        let mut last_err: Option<String> = None;

        for attempt in 0..retries {
            let result = self
                .http
                .get(&url)
                .headers(self.headers()?)
                .timeout(Duration::from_secs(90))
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_client_error() || status.is_server_error() {
                        let body = resp.text().await.unwrap_or_else(|_| "".to_string());
                        return Err(MuApiError::Api {
                            stage: "poll",
                            message: format!("poll failed [{}]: {body}", status.as_u16()),
                        });
                    }
                    return resp.json().await.map_err(|err| MuApiError::Transport {
                        stage: "poll",
                        message: format!("poll decode failed: {err}"),
                    });
                }
                Err(err) => {
                    if err.is_timeout() || err.is_connect() {
                        last_err = Some(err.to_string());
                        if attempt + 1 < retries {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    } else {
                        return Err(MuApiError::Transport {
                            stage: "poll",
                            message: err.to_string(),
                        });
                    }
                }
            }
        }

        Err(MuApiError::Api {
            stage: "poll",
            message: format!(
                "poll failed after {retries} retries: {}",
                last_err.unwrap_or_else(|| "unknown transport error".to_string())
            ),
        })
    }

    pub async fn poll(
        &self,
        request_id: &str,
        interval: f64,
        timeout: f64,
        label: Option<&str>,
    ) -> Result<Value, MuApiError> {
        let started = Instant::now();
        let mut last_status = String::new();
        let report_label = label.unwrap_or(request_id);

        while started.elapsed().as_secs_f64() < timeout {
            let data = self.fetch_result(request_id, 3).await?;
            let status = data
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();

            if !status.is_empty() && status != last_status {
                self.emitter.emit_status_change(report_label, &status);
                last_status = status.clone();
            }

            if status == "completed" || status == "succeeded" || status == "success" {
                return Ok(data);
            }
            if status == "failed" || status == "error" {
                return Err(MuApiError::Api {
                    stage: "poll",
                    message: format!("{report_label} failed: {data}"),
                });
            }

            tokio::time::sleep(Duration::from_secs_f64(interval)).await;
        }

        Err(MuApiError::Api {
            stage: "poll",
            message: format!("{report_label} timed out after {timeout}s"),
        })
    }

    pub async fn run(
        &self,
        endpoint: &str,
        payload: &Value,
        label: Option<&str>,
        interval: f64,
        timeout: f64,
    ) -> Result<Value, MuApiError> {
        let request_id = self.submit(endpoint, payload, 3).await?;
        self.poll(&request_id, interval, timeout, label.or(Some(endpoint)))
            .await
    }

    // TODO: decide explicit retry policy for 429/5xx parity-extension.
}
