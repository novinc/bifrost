use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use ha::api::{ClientMessage, ServerMessage};

use crate::error::{ApiError, ApiResult};

pub struct HomeAssistantWebSocket {
    pub name: String,
    pub socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    authenticated: bool,
}

impl HomeAssistantWebSocket {
    pub const fn new(name: String, socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            name,
            socket,
            authenticated: false,
        }
    }

    pub async fn auth(&mut self, token: String) -> ApiResult<()> {
        let json = serde_json::to_string(&ClientMessage::Auth {
            access_token: token,
        })?;
        let msg = Message::text(json);
        self.socket.send(msg).await?;
        let pkt = self
            .socket
            .next()
            .await
            .ok_or(ApiError::UnexpectedHaEof)??;

        let Message::Text(txt) = pkt else {
            log::error!("[{}] Received non-text message on websocket :(", self.name);
            return Err(ApiError::UnexpectedHaReply(pkt));
        };

        let raw_msg = serde_json::from_str::<ServerMessage>(&txt);
        let msg = raw_msg.map_err(|err| {
            log::error!(
                "[{}] Invalid websocket message: {:#?} [{}..]",
                self.name,
                err,
                &txt.chars().take(128).collect::<String>()
            );
            err
        })?;

        match msg {
            ServerMessage::AuthOk { ha_version } => {
                log::info!(
                    "Authenticated to home assistant with version {}",
                    ha_version.unwrap_or("unknown".to_string())
                );
                self.authenticated = true;
                Ok(())
            }
            ServerMessage::AuthInvalid { message } => {
                log::error!(
                    "Authentication invalid for home assistant websocket: {:?}",
                    message
                );
                Err(ApiError::HomeAssistantAuthenticateFailed)
            }
            _ => {
                log::error!(
                    "Invalid response to Authenticate request from home assistant websocket: {:?}",
                    msg
                );
                Err(ApiError::HomeAssistantAuthenticateFailed)
            }
        }
    }
}

impl Stream for HomeAssistantWebSocket
where
    Self: Unpin,
{
    type Item = Result<Message, tungstenite::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        WebSocketStream::poll_next(Pin::new(&mut self.socket), cx)
    }
}
