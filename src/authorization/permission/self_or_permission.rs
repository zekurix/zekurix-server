use std::marker::PhantomData;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::authorization::AuthorizationService;
use crate::error::Error;
use crate::identity::CurrentUserId;
use crate::router::ApiPath;
use crate::user::UserId;

use super::RequiredPermission;

#[allow(dead_code)]
pub struct SelfOrPermission<P>(PhantomData<P>);

impl<S, P> FromRequestParts<S> for SelfOrPermission<P>
where
    S: Send + Sync,
    AuthorizationService: FromRef<S>,
    P: RequiredPermission + Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUserId(current_user_id) =
            CurrentUserId::from_request_parts(parts, state).await?;

        let ApiPath(target_user_id) = ApiPath::<UserId>::from_request_parts(parts, state).await?;

        if current_user_id == target_user_id {
            return Ok(Self(PhantomData));
        }

        let service = AuthorizationService::from_ref(state);

        if !service
            .user_has_permission(current_user_id, P::PERMISSION)
            .await?
        {
            return Err(Error::Forbidden);
        }

        Ok(Self(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        extract::Extension,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    use crate::authorization::permission::RolesRead;
    use crate::authorization::{MockAuthorizationRepository, Permission};

    use super::*;

    #[derive(Clone)]
    struct TestState {
        authorization: AuthorizationService,
    }

    impl TestState {
        fn new(repository: MockAuthorizationRepository) -> Self {
            Self {
                authorization: AuthorizationService::new(Arc::new(repository)).unwrap(),
            }
        }
    }

    impl FromRef<TestState> for AuthorizationService {
        fn from_ref(state: &TestState) -> Self {
            state.authorization.clone()
        }
    }

    async fn protected_handler(_: SelfOrPermission<RolesRead>) -> StatusCode {
        StatusCode::NO_CONTENT
    }

    fn build_router(state: TestState, current_user_id: Option<UserId>) -> Router {
        let router = Router::new().route("/users/{id}", get(protected_handler));

        let router = if let Some(user_id) = current_user_id {
            router.layer(Extension(CurrentUserId(user_id)))
        } else {
            router
        };

        router.with_state(state)
    }

    async fn call(router: Router, target_user_id: UserId) -> StatusCode {
        router
            .oneshot(
                Request::builder()
                    .uri(format!("/users/{target_user_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn should_extract_when_target_user_is_current_user() {
        let user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_user_has_permission().times(0);

        let router = build_router(TestState::new(repository), Some(user_id));
        let status = call(router, user_id).await;

        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn should_extract_when_user_has_required_permission() {
        let current_user_id = UserId::new();
        let target_user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == current_user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Ok(true));

        let router = build_router(TestState::new(repository), Some(current_user_id));
        let status = call(router, target_user_id).await;

        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn should_reject_when_target_user_is_not_current_user_and_permission_is_missing() {
        let current_user_id = UserId::new();
        let target_user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == current_user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Ok(false));

        let router = build_router(TestState::new(repository), Some(current_user_id));
        let status = call(router, target_user_id).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn should_reject_when_current_user_id_is_missing() {
        let target_user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_user_has_permission().times(0);

        let router = build_router(TestState::new(repository), None);
        let status = call(router, target_user_id).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn should_propagate_permission_check_error() {
        let current_user_id = UserId::new();
        let target_user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == current_user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let router = build_router(TestState::new(repository), Some(current_user_id));
        let status = call(router, target_user_id).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
