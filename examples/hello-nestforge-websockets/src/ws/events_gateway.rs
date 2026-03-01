use nestforge::{
    handle_websocket_microservice_message, Message, WebSocket, WebSocketContext,
    WebSocketGateway,
};

use crate::app_config::AppConfig;
use crate::ws::WsPatterns;

pub struct EventsGateway;

impl WebSocketGateway for EventsGateway {
    fn on_connect(
        &self,
        ctx: WebSocketContext,
        mut socket: WebSocket,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> {
        Box::pin(async move {
            let app_name = ctx
                .resolve::<AppConfig>()
                .map(|config| config.app_name.clone())
                .unwrap_or_else(|_| "NestForge WebSockets".to_string());
            let patterns = ctx.resolve::<WsPatterns>().ok();

            let _ = socket
                .send(Message::Text(
                    format!(
                        "connected:{app_name}; send {{\"kind\":\"message\",\"pattern\":\"app.info\",\"payload\":null,\"metadata\":{{}}}}"
                    )
                    .into(),
                ))
                .await;

            while let Some(Ok(message)) = socket.recv().await {
                if matches!(message, Message::Close(_)) {
                    break;
                }

                if let Some(patterns) = patterns.as_ref() {
                    match handle_websocket_microservice_message(&ctx, patterns.registry(), message).await
                    {
                        Ok(Some(response)) => {
                            let _ = socket.send(response).await;
                        }
                        Ok(None) => {}
                        Err(err) => {
                            let _ = socket
                                .send(Message::Text(format!("error:{err}").into()))
                                .await;
                        }
                    }
                }
            }
        })
    }
}
