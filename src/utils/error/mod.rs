pub use cloud_error::{CloudError, CloudResult, IntoCloudError};
pub use error_kind::CloudErrorKind;
pub use error_kind::CloudErrorKind::*;

mod cloud_error;
mod error_kind;

#[macro_use]
pub mod macros;
