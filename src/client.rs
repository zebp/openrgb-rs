use crate::command::Command;
use crate::OpenRGBResult as Result;
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct OpenRGBClient {
    connection: TcpStream,
    name: String,
}

impl OpenRGBClient {
    pub async fn connect<A: ToSocketAddrs, S: Into<String>>(address: A, name: S) -> Result<Self> {
        let mut client = Self {
            connection: TcpStream::connect(address).await?,
            name: name.into(),
        };
        client.send_name().await.map(|_| client)
    }

    async fn send_name(&mut self) -> Result<()> {
        todo!()
    }

    async fn send_message(
        &mut self,
        command: Command,
        buffer: &[u8],
        device: Option<i32>,
    ) -> Result<()> {
        todo!()
    }
}
