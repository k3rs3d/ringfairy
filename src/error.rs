#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    ConfigError(#[from] clap::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Error: {0}")]
    StringError(String),
}
