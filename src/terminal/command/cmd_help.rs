use std::io::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cloud::Cloud;
use crate::sys_config::software_config::SoftwareConfig;
use crate::terminal::command_manager::CommandManager;

pub struct CmdHelp;

impl CommandManager for CmdHelp {
    async fn execute(_cloud: Arc<Mutex<Cloud>>, _args: Vec<&str>) -> Result<(), Error> {
        let config = SoftwareConfig::get();

        println!("{:?}", config);

        Ok(())
    }

    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}
