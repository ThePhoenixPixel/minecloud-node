use crate::types::service::Service;
use crate::types::services_local::LocalServices;
pub struct AllServices {
    local_services: LocalServices,
}

impl AllServices {
    pub async fn get_all(&self) -> Vec<Service> {
        let mut result: Vec<Service> = Vec::new();
        let local_services: Vec<Service> = self
            .local_services
            .get_all()
            .into_iter()
            .map(|s| s.clone())
            .collect();
        result.extend(local_services);
        result
    }
}
