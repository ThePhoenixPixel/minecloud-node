use std::io::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;

pub trait CommandManager {
    fn execute(
        cloud: Arc<RwLock<Cloud>>,
        args: Vec<&str>,
    ) -> impl std::future::Future<Output = Result<(), Error>>;
    fn tab_complete(args: Vec<&str>) -> Vec<String>;
}
