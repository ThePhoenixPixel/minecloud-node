#[macro_export]
macro_rules! error {
    // Variante ohne Quelle
    ($kind:expr) => {
        crate::utils::error::CloudError {
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
        crate::utils::error::CloudError {
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