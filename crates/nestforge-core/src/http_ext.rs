use std::fmt::Display;

use crate::HttpException;

pub trait ResultHttpExt<T, E> {
    fn or_bad_request(self) -> Result<T, HttpException>;
}

impl<T, E> ResultHttpExt<T, E> for Result<T, E>
where
    E: Display,
{
    fn or_bad_request(self) -> Result<T, HttpException> {
        self.map_err(|_err| HttpException::bad_request("invalid request"))
    }
}

pub trait OptionHttpExt<T> {
    fn or_not_found(self, message: impl Into<String>) -> Result<T, HttpException>;
    fn or_not_found_id(self, resource: &str, id: impl Display) -> Result<T, HttpException>;
}

impl<T> OptionHttpExt<T> for Option<T> {
    fn or_not_found(self, message: impl Into<String>) -> Result<T, HttpException> {
        self.ok_or_else(|| HttpException::not_found(message.into()))
    }

    fn or_not_found_id(self, resource: &str, id: impl Display) -> Result<T, HttpException> {
        self.ok_or_else(|| {
            HttpException::not_found(format!("{} with id {} not found", resource, id))
        })
    }
}
