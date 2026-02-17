pub use cloud_error::{CloudError, CloudResult, IntoCloudError};
pub use error_kind::CloudErrorKind;
pub use error_kind::CloudErrorKind::*;

mod error_kind;
mod cloud_error;

#[macro_use]
pub mod macros;



