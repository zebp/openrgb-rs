use crate::command::Command;
use crate::{
    network::{connection::OpenRGBConnection, packet::*},
    types::{OpenRGBColor, OpenRGBDevice},
    OpenRGBError, OpenRGBMode, OpenRGBResult,
};
use async_trait::async_trait;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
};

pub struct OpenRGBClient {
    connection: TcpStream,
    name: String,
    devices: Vec<OpenRGBDevice>,
}

impl OpenRGBClient {
    pub async fn connect<A: ToSocketAddrs, S: Into<String>>(
        address: A,
        name: S,
    ) -> OpenRGBResult<Self> {
        let mut client = Self {
            connection: TcpStream::connect(address).await?,
            name: name.into(),
            devices: vec![], // Assume a
        };

        client.send_name().await.map(|_| client)
    }

    async fn send_name(&mut self) -> OpenRGBResult<()> {
        let packet = SetClientNamePacket::new(self.name.clone());
        Self::send_packet(&mut self.connection, packet, None).await?;
        self.connection.flush().await?;
        Ok(())
    }

    /// Gets the number of devices that OpenRGB can control.
    pub async fn get_device_count(&mut self) -> OpenRGBResult<usize> {
        Self::send_command(&mut self.connection, Command::RequestControllerCount, None).await?;
        let count = match Self::read_packet(&mut self.connection).await? {
            OpenRGBPackets::RequestControllerCount(packet) => packet.count as usize,
            _ => todo!(),
        };
        Ok(count)
    }

    /// Requests device data from OpenRGB, this is just a representation of the device at the time it was requested.
    /// Changes to the device will not be tracked in this value.
    pub async fn get_device(&mut self, device_id: usize) -> OpenRGBResult<OpenRGBDevice> {
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

        // Store a copy of the device so we can use device info for later calls.
        if self.devices.len() <= device_id as usize {
            let mut devices = Vec::new();
            let mut inserted_new_device = false;

            for (id, old_device) in self.devices.iter().enumerate() {
                devices.push(if id == device_id {
                    inserted_new_device = true;
                    device.clone()
                } else {
                    old_device.clone()
                });
            }

            if !inserted_new_device {
                devices.push(device.clone())
            }

            self.devices = devices;
        } else {
            self.devices.insert(device_id as usize, device.clone());
        }

        Ok(device)
    }

    /// Sets the device into the "custom" mode, which will often be mode `0`.
    pub async fn set_custom_mode(&mut self, device_id: usize) -> OpenRGBResult<()> {
        Self::send_command(
            &mut self.connection,
            Command::SetCustomMode,
            Some(device_id),
        )
        .await
    }

    /// Sets the color of all leds from `0` to the length of the provided colors.
    ///
    /// ## Example:
    /// ```rust
    /// # use tokio::net::ToSocketAddrs;
    /// # use openrgb::*;
    /// # #[tokio::test]
    /// # async fn test() -> OpenRGBResult<()> {
    /// let client = OpenRGBClient::connect("0.0.0.0:6742", "Example").await?;
    /// let device = client.get_device(0).await?;
    /// let colors = vec![(255, 0, 0); device.colors.len()];
    /// client.update_leds(0, &colors).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_leds(
        &mut self,
        device_id: usize,
        colors: &[OpenRGBColor],
    ) -> OpenRGBResult<()> {
        let device = self
            .devices
            .get(device_id)
            .ok_or_else(|| OpenRGBError::InvalidId(device_id))?;

        if colors.len() > device.colors.len() {
            return Err(OpenRGBError::InvalidColorAmount(
                device.colors.len(),
                colors.len(),
            ));
        }

        let packet = UpdateLedsPacket::new(colors.to_vec());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    /// Sets all leds in the zone to their provided color.
    pub async fn update_zone_leds(
        &mut self,
        device_id: usize,
        zone_id: usize,
        colors: &[OpenRGBColor],
    ) -> OpenRGBResult<()> {
        let zone = self
            .devices
            .get(device_id)
            .ok_or_else(|| OpenRGBError::InvalidId(device_id))?
            .zones
            .get(zone_id)
            .ok_or_else(|| OpenRGBError::InvalidId(zone_id))?;

        if colors.len() > zone.leds_count as usize {
            return Err(OpenRGBError::InvalidColorAmount(
                zone.leds_count as usize,
                colors.len(),
            ));
        }

        let packet = UpdateZoneLedsPacket::new(zone_id, colors.to_vec());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    /// Updates the color of a single led.
    pub async fn update_single_led(
        &mut self,
        device_id: usize,
        led_id: usize,
        color: OpenRGBColor,
    ) -> OpenRGBResult<()> {
        let device = self
            .devices
            .get(device_id)
            .ok_or_else(|| OpenRGBError::InvalidId(device_id))?;

        if led_id >= device.leds.len() {
            return Err(OpenRGBError::InvalidId(led_id));
        }

        let packet = UpdateSingleLedPacket::new(led_id, color);
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    /// Updates the mode that the device is using and then switch to it.
    pub async fn update_mode(&mut self, device_id: usize, mode: &OpenRGBMode) -> OpenRGBResult<()> {
        let device = self
            .devices
            .get(device_id)
            .ok_or_else(|| OpenRGBError::InvalidId(device_id))?;
        let mode_id = device
            .modes
            .iter()
            .enumerate()
            .filter(|(_, dev_mode)| dev_mode.name == mode.name)
            .map(|(id, _)| id)
            .next()
            .ok_or_else(|| OpenRGBError::InvalidMode(mode.name.clone()))?;

        let packet = UpdateModePacket::new(mode_id, mode.clone());
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }

    pub async fn resize_zone(
        &mut self,
        device_id: usize,
        zone_id: usize,
        new_size: u32,
    ) -> OpenRGBResult<()> {
        self.devices
            .get(device_id)
            .ok_or_else(|| OpenRGBError::InvalidId(device_id))?
            .zones
            .get(zone_id)
            .ok_or_else(|| OpenRGBError::InvalidId(zone_id))?;

        let packet = ResizeZonePacket::new(zone_id, new_size);
        Self::send_packet(&mut self.connection, packet, Some(device_id)).await
    }
}

#[async_trait]
impl OpenRGBConnection for OpenRGBClient {}
