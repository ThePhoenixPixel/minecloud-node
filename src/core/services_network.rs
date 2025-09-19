use crate::core::service::Service;

pub struct NetworkServices {
    //services: Vec<Service>,
}

impl NetworkServices {
    pub fn new() -> NetworkServices {
        NetworkServices {
           // services: Vec::new(),
        }
    }

    pub async fn get_all(&self) -> Vec<&Service> {
        Vec::new()
    }

}
