use std::sync::Arc;

use adapter::repository::space::SpaceRepositoryImpl;
use adapter::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use kernel::repository::space::SpaceRepository;
use kernel::repository::health::HealthCheckRepository;

#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    space_repository: Arc<dyn SpaceRepository>,
}

impl AppRegistry {
    pub fn new(pool: ConnectionPool) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let space_repository = Arc::new(SpaceRepositoryImpl::new(pool.clone()));
        Self {
            health_check_repository,
            space_repository,
        }
    }

    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    pub fn space_repository(&self) -> Arc<dyn SpaceRepository> {
        self.space_repository.clone()
    }
}