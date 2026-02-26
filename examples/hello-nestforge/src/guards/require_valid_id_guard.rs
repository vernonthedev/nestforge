nestforge::guard!(RequireValidIdGuard, |ctx| {
    let path = ctx.uri.path();
    let has_zero_id = path.ends_with("/0") || path.contains("/0/");

    if has_zero_id {
        return Err(nestforge::HttpException::bad_request(
            "id must be greater than 0",
        ));
    }

    Ok(())
});
