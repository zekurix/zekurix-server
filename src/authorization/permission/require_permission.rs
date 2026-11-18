use std::marker::PhantomData;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::authorization::AuthorizationService;
use crate::error::Error;
use crate::identity::CurrentUserId;

use super::RequiredPermission;

#[allow(dead_code)]
pub struct RequirePermission<P>(PhantomData<P>);

impl<S, P> FromRequestParts<S> for RequirePermission<P>
where
    S: Send + Sync,
    AuthorizationService: FromRef<S>,
    P: RequiredPermission + Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUserId(user_id) = CurrentUserId::from_request_parts(parts, state).await?;

        let service = AuthorizationService::from_ref(state);

        if !service.user_has_permission(user_id, P::PERMISSION).await? {
            return Err(Error::Forbidden);
        }

        Ok(Self(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::Request;

    use crate::authorization::permission::RolesRead;
    use crate::authorization::{MockAuthorizationRepository, Permission};
    use crate::user::UserId;

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

    fn request_parts_with_current_user(user_id: UserId) -> Parts {
        let mut request = Request::builder().body(()).unwrap();
        request.extensions_mut().insert(CurrentUserId(user_id));

        let (parts, _) = request.into_parts();
        parts
    }

    #[tokio::test]
    async fn should_extract_when_user_has_required_permission() {
        let user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Ok(true));

        let state = TestState::new(repository);
        let mut parts = request_parts_with_current_user(user_id);

        let result = RequirePermission::<RolesRead>::from_request_parts(&mut parts, &state).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_when_user_does_not_have_required_permission() {
        let user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Ok(false));

        let state = TestState::new(repository);
        let mut parts = request_parts_with_current_user(user_id);

        let result = RequirePermission::<RolesRead>::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(Error::Forbidden)));
    }

    #[tokio::test]
    async fn should_reject_when_current_user_id_is_missing() {
        let mut repository = MockAuthorizationRepository::new();
        repository.expect_user_has_permission().times(0);

        let state = TestState::new(repository);
        let request = Request::builder().body(()).unwrap();
        let (mut parts, _) = request.into_parts();

        let result = RequirePermission::<RolesRead>::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(Error::AuthenticationRequired)));
    }

    #[tokio::test]
    async fn should_propagate_permission_check_error() {
        let user_id = UserId::new();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == Permission::RolesRead
            })
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let state = TestState::new(repository);
        let mut parts = request_parts_with_current_user(user_id);

        let result = RequirePermission::<RolesRead>::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }
}
