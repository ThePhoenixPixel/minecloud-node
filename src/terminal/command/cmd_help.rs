use std::io::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::config::software_config::SoftwareConfig;
use crate::terminal::command_manager::CommandManager;

pub struct CmdHelp;

impl CommandManager for CmdHelp {
    async fn execute(_cloud: Arc<RwLock<Cloud>>, _args: Vec<&str>) -> Result<(), Error> {
        let config = SoftwareConfig::get();

        println!("{:?}", config);

        Ok(())
    }

    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}
