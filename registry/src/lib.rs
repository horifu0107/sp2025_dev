use std::sync::Arc;

use adapter::repository::space::SpaceRepositoryImpl;
use adapter::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use adapter::repository::auth::AuthRepositoryImpl;
use adapter::repository::user::UserRepositoryImpl;

use adapter::redis::RedisClient;
use kernel::repository::health::HealthCheckRepository;
use kernel::repository::space::SpaceRepository;
use kernel::repository::auth::AuthRepository;
use kernel::repository::user::UserRepository;
use shared::config::AppConfig;

#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    space_repository: Arc<dyn SpaceRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl AppRegistry {
    pub fn new(pool: ConnectionPool,
        redis_client: Arc<RedisClient>,
        app_config: AppConfig,
    ) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(pool.clone()));
        let space_repository = Arc::new(SpaceRepositoryImpl::new(pool.clone()));
        let auth_repository = Arc::new(AuthRepositoryImpl::new(
            pool.clone(),
            redis_client.clone(),
            app_config.auth.ttl,
        ));
        let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));

        Self {
            health_check_repository,
            space_repository,
            auth_repository,
            user_repository,
        }
    }

    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    pub fn space_repository(&self) -> Arc<dyn SpaceRepository> {
        self.space_repository.clone()
    }
     pub fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }

    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }
}
