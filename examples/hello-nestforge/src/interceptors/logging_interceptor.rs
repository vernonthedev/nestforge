nestforge::interceptor!(LoggingInterceptor, |ctx, req, next| {
    let started = std::time::Instant::now();
    let response = (next)(req).await;
    println!(
        "[nestforge] {} {} - {}ms",
        ctx.method,
        ctx.uri,
        started.elapsed().as_millis()
    );
    response
});
