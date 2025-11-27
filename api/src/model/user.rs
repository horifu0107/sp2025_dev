use derive_new::new;
use garde::Validate;
use kernel::model::{
    id::UserId,
    role::Role,
    user::{
        event::{CreateUser, UpdateUserPassword, UpdateUserRole},
        User,
    },
};
use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[derive(Serialize, Deserialize, VariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleName {
    Admin,
    User,
}

impl From<Role> for RoleName {
    fn from(value: Role) -> Self {
        match value {
            Role::Admin => Self::Admin,
            Role::User => Self::User,
        }
    }
}

impl From<RoleName> for Role {
    fn from(value: RoleName) -> Self {
        match value {
            RoleName::Admin => Self::Admin,
            RoleName::User => Self::User,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    pub items: Vec<UserResponse>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_id: UserId,
    pub user_name: String,
    pub email: String,
    pub role: RoleName,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        let User {
            user_id,
            user_name,
            email,
            role,
        } = value;
        Self {
            user_id,
            user_name,
            email,
            role: RoleName::from(role),
        }
    }
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPasswordRequest {
    #[garde(length(min = 1))]
    current_password: String,
    #[garde(length(min = 1))]
    new_password: String,
}

#[derive(new)]
pub struct UpdateUserPasswordRequestWithUserId(UserId, UpdateUserPasswordRequest);
impl From<UpdateUserPasswordRequestWithUserId> for UpdateUserPassword {
    fn from(value: UpdateUserPasswordRequestWithUserId) -> Self {
        let UpdateUserPasswordRequestWithUserId(
            user_id,
            UpdateUserPasswordRequest {
                current_password,
                new_password,
            },
        ) = value;
        UpdateUserPassword {
            user_id,
            current_password,
            new_password,
        }
    }
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[garde(length(min = 1))]
    user_name: String,
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

impl From<CreateUserRequest> for CreateUser {
    fn from(value: CreateUserRequest) -> Self {
        let CreateUserRequest {
            user_name,
            email,
            password,
        } = value;
        Self {
            user_name,
            email,
            password,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequest {
    role: RoleName,
}

#[derive(new)]
pub struct UpdateUserRoleRequestWithUserId(UserId, UpdateUserRoleRequest);
impl From<UpdateUserRoleRequestWithUserId> for UpdateUserRole {
    fn from(value: UpdateUserRoleRequestWithUserId) -> Self {
        let UpdateUserRoleRequestWithUserId(user_id, UpdateUserRoleRequest { role }) = value;
        Self {
            user_id,
            role: Role::from(role),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceOwner {
    pub owner_id: UserId,
    pub owner_name: String,
}

impl From<kernel::model::user::SpaceOwner> for SpaceOwner {
    fn from(value: kernel::model::user::SpaceOwner) -> Self {
        let kernel::model::user::SpaceOwner { owner_id, owner_name } = value;
        Self { owner_id, owner_name }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationUser {
    pub user_id: UserId,
    pub user_name: String,
}

impl From<kernel::model::user::ReservationUser> for ReservationUser {
    fn from(value: kernel::model::user::ReservationUser) -> Self {
        let kernel::model::user::ReservationUser { user_id, user_name } = value;
        Self { user_id, user_name }
    }
}