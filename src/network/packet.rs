use crate::{
    command::Command,
    io::{AsyncOpenRGBReadExt, AsyncOpenRGBWriteExt, OpenRGBSendable},
    types::OpenRGBDevice,
    OpenRGBResult,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub enum OpenRGBPackets {
    SetClientName(SetClientNamePacket),
    RequestControllerCount(RequestControllerCountPacket),
    RequestControllerData(RequestControllerDataPacket),
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
