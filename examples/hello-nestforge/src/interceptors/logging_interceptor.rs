nestforge::interceptor!(LoggingInterceptor, |ctx, req, next| {
    let started = std::time::Instant::now();
    let response = (next)(req).await;
    let request_id = ctx.request_id.as_deref().unwrap_or("unknown").to_string();
    nestforge::framework_log_event(
        "example_interceptor_log",
        &[
            ("request_id", request_id),
            ("method", ctx.method.to_string()),
            ("path", ctx.uri.path().to_string()),
            ("duration_ms", started.elapsed().as_millis().to_string()),
            ("status", response.status().as_u16().to_string()),
        ],
    );
    response
});
