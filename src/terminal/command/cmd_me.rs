use std::io::{Error, ErrorKind};
use std::sync::Arc;
use sysinfo::{Pid, System};
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::log_info;
use crate::terminal::command_manager::CommandManager;

pub struct CmdMe;

impl CommandManager for CmdMe {
    async fn execute(_cloud: Arc<RwLock<Cloud>>, _args: Vec<&str>) -> Result<(), Error> {
        let pid = Pid::from_u32(std::process::id());

        match System::new().process(pid) {
            Some(process) => print_info(&process),
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to get the System Info".to_string(),
                ));
            }
        };

        Ok(())
    }

    fn tab_complete(_args: Vec<&str>) -> Vec<String> {
        todo!()
    }
}

fn print_info(process: &sysinfo::Process) {
    log_info!("------------>Cloud Info<------------");
    log_info!("Cpu: {:.2}%", process.cpu_usage());
    log_info!("Ram {} Bytes", process.memory());
    log_info!("------------------------------------");
}
