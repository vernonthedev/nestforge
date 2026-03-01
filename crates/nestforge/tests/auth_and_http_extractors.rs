use nestforge::{auth_guard, role_guard, Cookies, Headers, RequestContext};

auth_guard!(RequireAuthGuard);
role_guard!(RequireAdminGuard, "admin");

#[test]
fn auth_guard_rejects_unauthenticated_requests() {
    let ctx = RequestContext {
        method: axum::http::Method::GET,
        uri: "/secure".parse().expect("uri should parse"),
        request_id: None,
        auth_identity: None,
    };

    let result = <RequireAuthGuard as nestforge::Guard>::can_activate(
        &RequireAuthGuard,
        &ctx,
    );
    assert!(result.is_err());
}

#[test]
fn headers_and_cookies_wrappers_are_constructible() {
    let mut header_map = axum::http::HeaderMap::new();
    header_map.insert("x-demo", "yes".parse().expect("header should parse"));
    let headers = Headers(header_map);
    assert_eq!(
        headers
            .get("x-demo")
            .and_then(|value| value.to_str().ok()),
        Some("yes")
    );

    let cookies = Cookies::new([("session", "token")]);
    assert_eq!(cookies.get("session"), Some("token"));
}

#[test]
fn role_guard_rejects_authenticated_users_without_required_role() {
    let ctx = RequestContext {
        method: axum::http::Method::GET,
        uri: "/admin".parse().expect("uri should parse"),
        request_id: None,
        auth_identity: Some(std::sync::Arc::new(
            nestforge::AuthIdentity::new("demo-user").with_roles(["viewer"]),
        )),
    };

    let result = <RequireAdminGuard as nestforge::Guard>::can_activate(
        &RequireAdminGuard,
        &ctx,
    );
    assert!(result.is_err());
}
