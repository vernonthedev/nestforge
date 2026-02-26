use nestforge::{Guard, HttpException, RequestContext};

#[derive(Default)]
pub struct AllowAllGuard;

impl Guard for AllowAllGuard {
    fn can_activate(&self, _ctx: &RequestContext) -> Result<(), HttpException> {
        Ok(())
    }
}
