use database_manager::{DatabaseController, Table, TableDerive};
use database_manager::types::{DBDatetime, DBText, DBUInt, DBVarChar, DbError, DbResult, Filter, QueryFilters, Value};
use uuid::Uuid;

use crate::config::CloudConfig;
use crate::database::DBTools;
use crate::types::{Service, ServiceRef};

#[derive(TableDerive, Debug, Clone, Default)]
#[table_name("t_services")]
pub struct TableServices {
    #[primary_key]
    #[auto_increment]
    id: DBUInt,                  // service ID
    created_at: DBDatetime,      // format -> YYYY-MM-DD HH:MM:SS

    uuid: DBVarChar,
    name: DBText,
    typ: DBText,
    node: DBText,
    task: DBText,
    status: DBText,
    server_listener: DBText,
    plugin_listener: DBText,

    #[nullable]
    started_at: Option<DBDatetime>,         // format -> YYYY-MM-DD HH:MM:SS

    #[nullable]
    stopped_at: Option<DBDatetime>,         // format -> YYYY-MM-DD HH:MM:SS
}

impl TableServices {
    async fn new_from_service(service: &ServiceRef) -> Self {
        let service = service.read().await;
        TableServices {
            id: Default::default(),
            created_at: DBDatetime::get_now(),
            uuid: DBTools::uuid_to_varchar(&service.get_id()),
            name: DBText::from(service.get_name()),
            typ: DBText::from(service.get_software_name().get_server_type().to_string()),
            node: DBText::from(service.get_start_node()),
            task: DBText::from(service.get_task().get_name()),
            status: DBText::from(service.get_status().to_string()),
            server_listener: DBText::from(service.get_server_listener().to_string()),
            plugin_listener: DBText::from(service.get_plugin_listener().to_string()),
            started_at: service.get_started_at_to_string().map(|s| DBDatetime::from(s)),
            stopped_at: service.get_stopped_at_to_string().map(|s| DBDatetime::from(s)),
        }
    }
    
    pub async fn create<M: DatabaseController>(db: &M, service: &ServiceRef) -> DbResult<()> {
        let ts = Self::new_from_service(service).await;
        db.insert(Self::table_name(), &Self::to_row(&ts)).await?;
        Ok(())
    }
    
    pub async fn update<M: DatabaseController>(db: &M, service: &ServiceRef) -> DbResult<()> {
        let f = QueryFilters::new().add(Filter::eq("uuid", DBTools::uuid_to_value(&service.get_id().await)));
        let row = db.query_one(Self::table_name(), &f)
            .await?
            .ok_or(DbError::NotFound(String::from("Service not found")))?;

        let table_service      = Self::from_row(&row)?;
        let mut new_service    = Self::new_from_service(service).await;
        new_service.id                      = table_service.id;
        new_service.created_at              = table_service.created_at;
        
        let f = QueryFilters::new().add(Filter::eq("id", Value::UInt(table_service.id)));
        db.update(Self::table_name(), &f, &Self::to_row(&new_service)).await?;
        Ok(())
    }
    
    pub async fn delete<M: DatabaseController>(db: &M, service_uuid: &Uuid) -> DbResult<()> {
        let f = QueryFilters::new().add(Filter::eq("uuid", DBTools::uuid_to_value(&service_uuid)));
        db.delete(Self::table_name(), &f).await?;
        Ok(())
    }

    pub async fn delete_others<M: DatabaseController>(db: &M, service_list: &Vec<Service>, config: &CloudConfig) -> DbResult<()> {
        let mut f = QueryFilters::new();
        f.add_filter(Filter::eq("node", Value::from(config.get_name())));
        for s in service_list {
            f.add_filter(Filter::eq("uuid", DBTools::uuid_to_value(&s.get_id())));
        }
        db.delete(Self::table_name(), &f).await?;
        Ok(())
    }
    
}

