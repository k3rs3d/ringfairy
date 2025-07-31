use crate::error::Error;

pub fn format_error(err: Error) -> String {
    match err {
        Error::TemplateError(error) => error.to_string(),
        Error::HttpError(error) => error.to_string(),
        Error::ConfigError(error) => error.to_string(),
        Error::IOError(error) => error.to_string(),
        Error::Utf8Error(error) => error.to_string(),
        Error::JsonError(error) => error.to_string(),
        Error::TOMLError(error) => error.to_string(),
        Error::StringError(error) => error.to_string(),
    }
}
