use crate::utils::error_kind::CloudErrorKind;
use std::{error::Error, fmt};
#[derive(Debug)]
pub struct CloudError {
    pub kind: CloudErrorKind,
    pub message: String,
    pub source_message: Option<String>,
    pub source: Option<Box<dyn Error + Send + Sync>>,
    pub file: &'static str,
    pub line: u32,
}

impl fmt::Display for CloudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref source_msg) = self.source_message {
            write!(
                f,
                "Error [{}]: {} | Source: {} (at {}:{})",
                self.kind.code(),
                self.message,
                source_msg,
                self.file,
                self.line
            )
        } else {
            write!(
                f,
                "Error [{}]: {} (at {}:{})",
                self.kind.code(),
                self.message,
                self.file,
                self.line
            )
        }
    }
}

impl Error for CloudError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

impl From<Box<dyn Error + Send + Sync>> for CloudError {
    fn from(err: Box<dyn Error + Send + Sync>) -> Self {
        CloudError {
            kind: CloudErrorKind::Internal,
            message: err.to_string(),
            source_message: Some(err.to_string()),
            source: Some(err),
            file: file!(),
            line: line!(),
        }
    }
}

impl From<Box<dyn Error>> for CloudError {
    fn from(err: Box<dyn Error>) -> Self {
        let msg = err.to_string();
        CloudError {
            kind: CloudErrorKind::Internal,
            message: msg.clone(),
            source_message: Some(msg),
            source: None,
            file: file!(),
            line: line!(),
        }
    }
}

pub trait IntoCloudError<T> {
    fn into_cloud_error(self, kind: CloudErrorKind) -> Result<T, CloudError>;
}

impl<T, E> IntoCloudError<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn into_cloud_error(self, kind: CloudErrorKind) -> Result<T, CloudError> {
        self.map_err(|e| {
            let msg = e.to_string();
            CloudError {
                kind: kind.clone(),
                message: msg.clone(),
                source_message: Some(msg),
                source: Some(Box::new(e)),
                file: file!(),
                line: line!(),
            }
        })
    }
}

#[macro_export]
macro_rules! error {
    // Variante ohne Quelle
    ($kind:expr) => {
        CloudError {
            kind: $kind,
            message: "Cant Load from Language".to_string(),
            source_message: None,
            source: None,
            file: file!(),
            line: line!(),
        }
    };
    // Variante mit Quelle
    ($kind:expr, $src:expr) => {{
        let source_msg = $src.to_string();
        CloudError {
            kind: $kind,
            message: "Cant Load from Language".to_string(),
            source_message: Some(source_msg.clone()),
            source: Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                source_msg,
            ))),
            file: file!(),
            line: line!(),
        }
    }};
}
