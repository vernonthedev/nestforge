use std::time::Instant;

use nestforge::{Interceptor, NextFn, NextFuture, RequestContext};

#[derive(Default)]
pub struct LoggingInterceptor;

impl Interceptor for LoggingInterceptor {
    fn around(
        &self,
        ctx: RequestContext,
        req: axum::extract::Request,
        next: NextFn,
    ) -> NextFuture {
        Box::pin(async move {
            let started = Instant::now();
            let response = (next)(req).await;
            println!(
                "[nestforge] {} {} - {}ms",
                ctx.method,
                ctx.uri,
                started.elapsed().as_millis()
            );
            response
        })
    }
}
