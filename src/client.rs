use crate::command::Command;
use crate::{
    network::{packet::{OpenRGBPackets, SetClientNamePacket}, OpenRGBConnection},
    types::OpenRGBDevice,
    OpenRGBResult,
};
use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::{TcpStream, ToSocketAddrs}};

pub struct OpenRGBClient {
    connection: TcpStream,
    name: String,
}

impl OpenRGBClient {
    pub async fn connect<A: ToSocketAddrs, S: Into<String>>(
        address: A,
        name: S,
    ) -> OpenRGBResult<Self> {
        let mut client = Self {
            connection: TcpStream::connect(address).await?,
            name: name.into(),
        };
        client.send_name().await.map(|_| client)
    }

    async fn send_name(&mut self) -> OpenRGBResult<()> {
        let packet = SetClientNamePacket::new(self.name.clone());
        Self::send_packet(&mut self.connection, packet, None).await?;
        self.connection.flush().await?;
        println!("flushed");
        Ok(())
    }

    pub async fn get_devices(&mut self) -> OpenRGBResult<Vec<OpenRGBDevice>> {
        Self::send_command(&mut self.connection, Command::RequestControllerCount, None).await?;
        let count = match Self::read_packet(&mut self.connection).await? {
            OpenRGBPackets::RequestControllerCount(packet) => packet.count,
            _ => todo!(),
        };

        let mut devices = Vec::with_capacity(count as usize);

        for index in 0..count {
            Self::send_command(&mut self.connection, Command::RequestControllerData, Some(index)).await?;
            let device = match Self::read_packet(&mut self.connection).await? {
                OpenRGBPackets::RequestControllerData(packet) => packet.device,
                _ => todo!(),
            }; 

            devices.push(device);
        }
        
        Ok(devices)
    }


}

#[async_trait]
impl OpenRGBConnection for OpenRGBClient {}
