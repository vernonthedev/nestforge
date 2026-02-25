/* 
main.rs should stay super small.

Its job:
- load app modules/files
- boot the framework
*/

mod app_module;
mod controllers;
mod dto;
mod services;

use app_module::AppModule;
use nestforge::NestForgeFactory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*
    Framework boot using AppModule
    */
    NestForgeFactory::<AppModule>::create()?.listen(3000).await
}