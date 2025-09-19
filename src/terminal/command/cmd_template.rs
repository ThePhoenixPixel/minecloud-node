use std::io::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cloud::Cloud;
use crate::terminal::command_manager::CommandManager;

pub struct CmdTemplate;

impl CommandManager for CmdTemplate {
    async fn execute(_cloud: Arc<Mutex<Cloud>>, _args: Vec<&str>) -> Result<(), Error> {
        todo!()
    }

    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}
