use colored::{ColoredString, Colorize};
use std::io;
use std::io::{Error, ErrorKind, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::log_error;
use crate::terminal::command::cmd_help::CmdHelp;
use crate::terminal::command::cmd_me::CmdMe;
use crate::terminal::command::cmd_service::CmdService;
use crate::terminal::command::cmd_task::CmdTask;
use crate::terminal::command::cmd_template::CmdTemplate;
use crate::terminal::command_manager::CommandManager;

pub struct Cmd {
    prefix: ColoredString,
    cloud: Arc<RwLock<Cloud>>,
}

impl Cmd {
    pub fn new(prefix: &ColoredString, cloud: Arc<RwLock<Cloud>>) -> Cmd {
        Cmd {
            prefix: prefix.clone(),
            cloud,
        }
    }

    pub async fn start(&self) {
        //start the cmd system
        loop {
            // print the prefix
            print!(
                "{} ",
                ColoredString::from(format!("{} >>", &self.prefix)).blue()
            );

            // flush the buffer
            flush_buffer();

            // read line input from terminal
            let input = read_from_line();

            // trim the input
            let args: Vec<&str> = input.trim().split_whitespace().collect();

            let command = match args.first() {
                Some(command) => command.to_string(),
                None => String::new(),
            }
            .to_lowercase();

            // check ob stop or exit execute in the input the stop the cloud
            if command == "exit" || command == "stop" {
                break;
            }

            // execute the commands
            match Cmd::execute_command(self.cloud.clone(), command.as_str(), args).await {
                Ok(_) => {}
                Err(e) => log_error!("{}", e),
            }
        }
        let mut cloud = self.cloud.write().await;
        cloud.disable().await;
    }

    pub async fn execute_command(
        cloud: Arc<RwLock<Cloud>>,
        command: &str,
        args: Vec<&str>,
    ) -> Result<(), Error> {
        match command {
            "help" => CmdHelp::execute(cloud, args).await,
            "task" => CmdTask::execute(cloud, args).await,
            "service" => CmdService::execute(cloud, args).await,
            "template" => CmdTemplate::execute(cloud, args).await,
            "me" => CmdMe::execute(cloud, args).await,
            "" => Ok(()),
            _ => Err(Error::new(
                ErrorKind::Other,
                "Bitte gebe ein gültigen command an".to_string(),
            )),
        }
    }
}

fn flush_buffer() {
    match io::stdout().flush() {
        Ok(_) => return,
        Err(e) => {
            log_error!("Error by flushing the Buffer");
            log_error!("{}", e.to_string());
        }
    }
}

fn read_from_line() -> String {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input,
        Err(e) => {
            log_error!("Error by read the input");
            log_error!("{}", e.to_string());
            String::new()
        }
    }
}
