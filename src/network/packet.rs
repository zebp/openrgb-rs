use crate::{
    command::Command,
    io::{AsyncOpenRGBReadExt, AsyncOpenRGBWriteExt, OpenRGBSendable},
    types::OpenRGBDevice,
    OpenRGBColor, OpenRGBMode, OpenRGBResult,
};
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub enum OpenRGBPackets {
    SetClientName(SetClientNamePacket),
    RequestControllerCount(RequestControllerCountPacket),
    RequestControllerData(RequestControllerDataPacket),
    UpdateMode(UpdateModePacket),
    UpdateLeds(UpdateLedsPacket),
    UpdateZoneLeds(UpdateZoneLedsPacket),
    UpdateSingleLed(UpdateSingleLedPacket),
    ResizeZone(ResizeZonePacket),
    /// A packet that has no data other than it's id
    Command(Command),
}

pub trait OpenRGBPacket: Sync + OpenRGBSendable {
    fn command(&self) -> Command;
}

#[derive(Debug, Clone)]
pub struct SetClientNamePacket {
    pub name: String,
}

impl SetClientNamePacket {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl OpenRGBSendable for SetClientNamePacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_all(self.name.as_bytes()).await?;
        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;

        let name = String::from_utf8(buffer).unwrap();
        Ok(Self { name })
    }
}

impl OpenRGBPacket for SetClientNamePacket {
    fn command(&self) -> Command {
        Command::SetClientName
    }
}

#[derive(Debug, Clone)]
pub struct RequestControllerCountPacket {
    pub count: u32,
}

impl RequestControllerCountPacket {
    pub fn new(count: u32) -> Self {
        Self { count }
    }
}

#[async_trait]
impl OpenRGBSendable for RequestControllerCountPacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_u32_le(self.count).await?;
        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let count = reader.read_u32_le().await?;
        Ok(Self { count })
    }
}

impl OpenRGBPacket for RequestControllerCountPacket {
    fn command(&self) -> Command {
        Command::RequestControllerCount
    }
}

#[derive(Debug, Clone)]
pub struct RequestControllerDataPacket {
    pub device: OpenRGBDevice,
}

impl RequestControllerDataPacket {
    pub fn new(device: OpenRGBDevice) -> Self {
        Self { device }
    }
}

#[async_trait]
impl OpenRGBSendable for RequestControllerDataPacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        self.device.serialize(writer).await
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let device = OpenRGBDevice::deserialize(reader).await?;
        Ok(Self { device })
    }
}

impl OpenRGBPacket for RequestControllerDataPacket {
    fn command(&self) -> Command {
        Command::RequestControllerData
    }
}

#[derive(Debug, Clone)]
pub struct UpdateModePacket {
    pub mode_id: usize,
    pub mode: OpenRGBMode,
}

impl UpdateModePacket {
    pub fn new(mode_id: usize, mode: OpenRGBMode) -> Self {
        Self { mode_id, mode }
    }
}

#[async_trait]
impl OpenRGBSendable for UpdateModePacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.write_u32_le(self.mode_id as u32).await?;
        self.mode.serialize(&mut buffer).await?;

        writer.write_u32_le(buffer.len() as u32).await?;
        writer.write_all(&buffer).await?;

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self> {
        let _length = reader.read_u32_le().await?;
        let mode_id = reader.read_u32_le().await? as usize;
        let mode = OpenRGBMode::deserialize(reader).await?;
        Ok(Self::new(mode_id, mode))
    }
}

impl OpenRGBPacket for UpdateModePacket {
    fn command(&self) -> Command {
        Command::UpdateMode
    }
}

#[derive(Debug, Clone)]
pub struct UpdateLedsPacket {
    pub colors: Vec<OpenRGBColor>,
}

impl UpdateLedsPacket {
    pub fn new(colors: Vec<OpenRGBColor>) -> Self {
        Self { colors }
    }
}

#[async_trait]
impl OpenRGBSendable for UpdateLedsPacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.write_u16_le(self.colors.len() as u16).await?;

        for color in &self.colors {
            color.serialize(&mut buffer).await?
        }

        writer.write_u32_le(buffer.len() as u32).await?;
        writer.write_all(&buffer).await?;

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let _buffer_length = reader.read_u32_le().await?; // This is fucking stupid
        let length = reader.read_u16_le().await? as usize;
        let mut colors = Vec::with_capacity(length);

        for _ in 0..length {
            colors.push(OpenRGBColor::deserialize(reader).await?);
        }

        Ok(Self::new(colors))
    }
}

impl OpenRGBPacket for UpdateLedsPacket {
    fn command(&self) -> Command {
        Command::UpdateLeds
    }
}

#[derive(Debug, Clone)]
pub struct UpdateZoneLedsPacket {
    pub zone_id: usize,
    pub colors: Vec<OpenRGBColor>,
}

impl UpdateZoneLedsPacket {
    pub fn new(zone_id: usize, colors: Vec<OpenRGBColor>) -> Self {
        Self { zone_id, colors }
    }
}

#[async_trait]
impl OpenRGBSendable for UpdateZoneLedsPacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        let mut buffer = Vec::new();

        buffer.write_u32_le(self.zone_id as u32).await?;
        buffer.write_u16_le(self.colors.len() as u16).await?;

        for color in &self.colors {
            color.serialize(&mut buffer).await?
        }

        writer.write_u32_le(buffer.len() as u32).await?;
        writer.write_all(&buffer).await?;

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let _ = reader.read_u32_le().await?;
        let zone_id = reader.read_u32_le().await? as usize;
        let length = reader.read_u16_le().await? as usize;
        let mut colors = Vec::with_capacity(length);

        for _ in 0..length {
            colors.push(OpenRGBColor::deserialize(reader).await?);
        }

        Ok(Self::new(zone_id, colors))
    }
}

impl OpenRGBPacket for UpdateZoneLedsPacket {
    fn command(&self) -> Command {
        Command::UpdateZoneLeds
    }
}

#[derive(Debug, Clone)]
pub struct UpdateSingleLedPacket {
    pub led_id: usize,
    pub color: OpenRGBColor,
}

impl UpdateSingleLedPacket {
    pub fn new(led_id: usize, color: OpenRGBColor) -> Self {
        Self { led_id, color }
    }
}

#[async_trait]
impl OpenRGBSendable for UpdateSingleLedPacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_u32_le(self.led_id as u32).await?;
        self.color.serialize(writer).await?;
        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let led_id = reader.read_u32_le().await? as usize;
        let color = OpenRGBColor::deserialize(reader).await?;
        Ok(Self::new(led_id, color))
    }
}

impl OpenRGBPacket for UpdateSingleLedPacket {
    fn command(&self) -> Command {
        Command::UpdateSingleLed
    }
}

#[derive(Debug, Clone)]
pub struct ResizeZonePacket {
    pub zone_id: usize,
    pub new_size: u32,
}

impl ResizeZonePacket {
    pub fn new(zone_id: usize, new_size: u32) -> Self {
        Self { zone_id, new_size }
    }
}

#[async_trait]
impl OpenRGBSendable for ResizeZonePacket {
    type Output = Self;
    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_u32_le(self.zone_id as u32).await?;
        writer.write_u32_le(self.new_size).await?;
        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let zone_id = reader.read_u32_le().await? as usize;
        let new_size = reader.read_u32_le().await?;
        Ok(Self::new(zone_id, new_size))
    }
}

impl OpenRGBPacket for ResizeZonePacket {
    fn command(&self) -> Command {
        Command::ResizeZone
    }
}
