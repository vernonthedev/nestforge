use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use nestforge::{ApiSerializedResult, ResponseSerializer, Serialized};
use tower::ServiceExt;

#[derive(Clone)]
struct UserEntity {
    id: u64,
    email: String,
    password_hash: String,
}

#[derive(serde::Serialize)]
struct UserDto {
    id: u64,
    email: String,
}

struct UserSerializer;

impl ResponseSerializer<UserEntity> for UserSerializer {
    type Output = UserDto;

    fn serialize(value: UserEntity) -> Self::Output {
        let _ = value.password_hash;
        UserDto {
            id: value.id,
            email: value.email,
        }
    }
}

#[derive(Default)]
struct SerializerController;

#[nestforge::controller("/serializers")]
impl SerializerController {
    #[nestforge::get("/user")]
    async fn user() -> ApiSerializedResult<UserEntity, UserSerializer> {
        Ok(Serialized::new(UserEntity {
            id: 7,
            email: "alice@example.com".to_string(),
            password_hash: "secret".to_string(),
        }))
    }
}

#[derive(Default)]
struct SerializerModule;

#[nestforge::module(controllers = [SerializerController])]
impl SerializerModule {}

#[tokio::test]
async fn response_serializers_shape_handler_payloads() {
    let app = nestforge::NestForgeFactory::<SerializerModule>::create()
        .expect("factory")
        .into_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/serializers/user")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("json");

    assert_eq!(json["id"], 7);
    assert_eq!(json["email"], "alice@example.com");
    assert!(json.get("password_hash").is_none());
}
