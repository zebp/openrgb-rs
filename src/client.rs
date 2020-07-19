use crate::command::Command;
use crate::{
    network::{connection::OpenRGBConnection, packet::*},
    types::{OpenRGBColor, OpenRGBDevice},
    OpenRGBMode, OpenRGBResult,
};
use async_trait::async_trait;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
};

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
        Ok(())
    }

    pub async fn get_device_count(&mut self) -> OpenRGBResult<u32> {
        Self::send_command(&mut self.connection, Command::RequestControllerCount, None).await?;
        let count = match Self::read_packet(&mut self.connection).await? {
            OpenRGBPackets::RequestControllerCount(packet) => packet.count,
            _ => todo!(),
        };
        Ok(count)
    }

    pub async fn get_device(&mut self, device_id: u32) -> OpenRGBResult<OpenRGBDevice> {
        Self::send_command(
            &mut self.connection,
            Command::RequestControllerData,
            Some(device_id),
        )
        .await?;
        let device = match Self::read_packet(&mut self.connection).await? {
            OpenRGBPackets::RequestControllerData(packet) => packet.device,
            _ => todo!(),
        };

        Ok(device)
    }

    pub async fn set_custom_mode(&mut self, device_id: u32) -> OpenRGBResult<()> {
        Self::send_command(
            &mut self.connection,
            Command::SetCustomMode,
            Some(device_id),
        )
        .await
    }

    pub async fn update_leds(
        &mut self,
        device_id: u32,
        colors: &[OpenRGBColor],
    ) -> OpenRGBResult<()> {
        let packet = UpdateLedsPacket::new(colors.to_vec());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    pub async fn update_zone_leds(
        &mut self,
        device_id: u32,
        zone_id: usize,
        colors: &[OpenRGBColor],
    ) -> OpenRGBResult<()> {
        let packet = UpdateZoneLedsPacket::new(zone_id, colors.to_vec());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    pub async fn update_single_led(
        &mut self,
        device_id: u32,
        led_id: usize,
        color: OpenRGBColor,
    ) -> OpenRGBResult<()> {
        let packet = UpdateSingleLedPacket::new(led_id, color);
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    pub async fn update_mode(
        &mut self,
        device_id: u32,
        mode_id: usize,
        mode: &OpenRGBMode,
    ) -> OpenRGBResult<()> {
        let packet = UpdateModePacket::new(mode_id, mode.clone());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    pub async fn resize_zone(
        &mut self,
        device_id: u32,
        zone_id: usize,
        new_size: u32,
    ) -> OpenRGBResult<()> {
        let packet = ResizeZonePacket::new(zone_id, new_size);
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }
}

#[async_trait]
impl OpenRGBConnection for OpenRGBClient {}
