use std::sync::Arc;

use adapter::repository::space::SpaceRepositoryImpl;
use adapter::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use adapter::repository::auth::AuthRepositoryImpl;
use adapter::repository::user::UserRepositoryImpl;
use adapter::repository::reservation::ReservationRepositoryImpl;
use anyhow::Result;
use yup_oauth2::{
    read_application_secret, InstalledFlowAuthenticator, InstalledFlowReturnMethod
};
use tokio::sync::Mutex;
use shared::error::{AppError, AppResult};


use adapter::redis::RedisClient;
use kernel::repository::health::HealthCheckRepository;
use kernel::repository::space::SpaceRepository;
use kernel::repository::auth::AuthRepository;
use kernel::repository::user::UserRepository;
use kernel::repository::reservation::ReservationRepository;

use shared::config::AppConfig;

#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    space_repository: Arc<dyn SpaceRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
    reservation_repository: Arc<dyn ReservationRepository>,
}

const SECRET_PATH: &str =
    "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/client_secret_483730081753-qm9ujsmkcgfpag17j2iv618fspsjpgou.apps.googleusercontent.com.json";

const TOKEN_PATH: &str =
    "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/token.json";


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
        let reservation_repository = Arc::new(ReservationRepositoryImpl::new(pool.clone()));


        Self {
            health_check_repository,
            space_repository,
            auth_repository,
            user_repository,
            reservation_repository,
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

    pub fn reservation_repository(&self) -> Arc<dyn ReservationRepository> {
        self.reservation_repository.clone()
    }

    pub async fn google_access_token(&self) -> AppResult<String> {
        let secret_path = "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/client_secret_483730081753-qm9ujsmkcgfpag17j2iv618fspsjpgou.apps.googleusercontent.com.json";
        let token_path = "/Users/horikawafuka2/Documents/class_2025/sp/test_gmail/token.json";
    
        // let secret = yup_oauth2::read_application_secret(secret_path).await?;
        let secret = yup_oauth2::read_application_secret(secret_path)
            .await
            .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;
        
        let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
            .persist_tokens_to_disk(token_path)
            .build()
            .await?;
    
        let token = auth
            .token(&["https://www.googleapis.com/auth/gmail.send"])
            .await?;
        let access_token = token.token().unwrap().to_string();
        

        Ok(access_token)
    }
}
