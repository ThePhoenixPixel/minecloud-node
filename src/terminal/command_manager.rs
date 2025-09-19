use std::io::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cloud::Cloud;

pub trait CommandManager {
    fn execute(
        cloud: Arc<Mutex<Cloud>>,
        args: Vec<&str>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
    fn tab_complete(args: Vec<&str>) -> Vec<String>;
}
