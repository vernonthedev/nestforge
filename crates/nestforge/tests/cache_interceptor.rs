#![cfg(all(feature = "cache", feature = "redis"))]

use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Json,
};
use tower::ServiceExt;

static USER_LIST_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Default, Clone)]
struct UsersCachePolicy;

impl nestforge::CachePolicy for UsersCachePolicy {
    type Store = nestforge::InMemoryRedisStore;
}

#[derive(Default)]
struct CacheController;

#[nestforge::controller("/cache")]
impl CacheController {
    #[nestforge::get("/users")]
    async fn users() -> nestforge::ApiResult<usize> {
        let count = USER_LIST_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
        Ok(Json(count))
    }
}

#[nestforge::module(
    controllers = [CacheController],
    providers = [nestforge::InMemoryRedisStore::default()]
)]
struct CacheModule;

#[tokio::test]
async fn cache_interceptor_reuses_cached_get_responses() {
    USER_LIST_CALLS.store(0, Ordering::Relaxed);

    let app = nestforge::NestForgeFactory::<CacheModule>::create()
        .expect("factory")
        .use_interceptor::<nestforge::CacheInterceptor<UsersCachePolicy>>()
        .into_router();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/cache/users")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    let second = app
        .oneshot(
            Request::builder()
                .uri("/cache/users")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);

    let first_body = axum::body::to_bytes(first.into_body(), usize::MAX)
        .await
        .expect("first body");
    let second_body = axum::body::to_bytes(second.into_body(), usize::MAX)
        .await
        .expect("second body");

    assert_eq!(first_body, second_body);
    assert_eq!(USER_LIST_CALLS.load(Ordering::Relaxed), 1);
}
