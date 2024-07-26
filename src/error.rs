use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),
}