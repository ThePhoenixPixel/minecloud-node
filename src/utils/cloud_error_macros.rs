#[macro_export]
macro_rules! cloud_error {
    ($code:expr, $msg:expr) => {
        CloudError {
            code: $code,
            message: $msg.to_string(),
            source: None,
            file: file!(),
            line: line!(),
            func: function_path!(),
        }
    };
    ($code:expr, $msg:expr, $src:expr) => {
        CloudError {
            code: $code,
            message: $msg.to_string(),
            source: Some(Box::new($src)),
            file: file!(),
            line: line!(),
            func: function_path!(),
        }
    };
}

#[macro_export]
macro_rules! function_path {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3] // "f" abschneiden
    }};
}
