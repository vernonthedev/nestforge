use nestforge::{Message, WebSocket, WebSocketContext, WebSocketGateway};

use crate::app_config::AppConfig;

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

            let _ = socket
                .send(Message::Text(format!("connected:{app_name}").into()))
                .await;

            while let Some(Ok(message)) = socket.recv().await {
                match message {
                    Message::Text(text) => {
                        let _ = socket
                            .send(Message::Text(format!("echo:{text}").into()))
                            .await;
                    }
                    Message::Binary(bytes) => {
                        let _ = socket.send(Message::Binary(bytes)).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        })
    }
}
