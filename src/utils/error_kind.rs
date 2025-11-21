
#[derive(Debug, Clone)]
pub enum CloudErrorKind {
    /// Default
    Null,

    /// 1x.xxx system
    CantReadFileToString,
    
    /// 11.xxx Directory
    PathServe,

    /// 12.xxx network
    NextFreePortNotFound,


    /// 2x.xxx CloudSystem

    /// 21.xxx Task
    TaskNotFound,

    /// 22.xxx Template
    TemplateNotFound,

    /// 3x.xxx Service
    /// 31.xxx Not Found
    ServiceNotFound,
    CantCreateServiceFolder,
    CantCopyTemplateToNewServiceFolder,
    CantCreateSystemPluginPath,
    CantFindSystemPlugin,
    CantCopySystemPlugin,
    
    CantFindIPConfigFilePath,
    CantWriteIP,
    
    CantFindPortConfigFilePath,
    CantWritePort,
    
    CantCopySoftware,
    
    CantCreateSTDOUTFile,
    CantCreateSTDERRFile,
    
    CantStartServer,
    
    
    CantConvertServerFilePathToString,


    /// 9.xxx
    /// Internal System
    IoError,
    Internal,
}


impl CloudErrorKind {
    pub fn code(&self) -> u32 {
        match self {
            // 10.xxx Directory
            CloudErrorKind::NextFreePortNotFound            => 120001,


            // 2x.xxx CloudSystem

            // 21.xxx Task
            // 21.1xx NotFound
            CloudErrorKind::TaskNotFound                    => 210000,

            // 22.xxxx Template
            // 22.1.xx NotFound
            CloudErrorKind::TemplateNotFound                => 221000,

            // 3x.xxx Service
            // 30.1xx NotFound
            CloudErrorKind::ServiceNotFound                 => 310000,


            // 9.xxx
            // Internal System
            CloudErrorKind::IoError                         => 999999,
            CloudErrorKind::Internal                        => 900000,


            // Default
            _                                               => 0,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            CloudErrorKind::NextFreePortNotFound => "Next Free Port nicht gefunden",
            CloudErrorKind::TaskNotFound => "Task nicht gefunden",
            CloudErrorKind::ServiceNotFound => "Service nicht gefunden",
            CloudErrorKind::TemplateNotFound => "Template nicht gefunden",
            CloudErrorKind::IoError => "IO Fehler",
            CloudErrorKind::Internal => "Interner Fehler",
            _ => "NUll",
        }
    }

}

