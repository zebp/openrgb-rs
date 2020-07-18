use crate::{
    command::Command,
    io::{AsyncOpenRGBReadExt, AsyncOpenRGBWriteExt, OpenRGBSendable},
    OpenRGBResult,
};
use async_trait::async_trait;
use std::convert::TryFrom;

#[async_trait]
pub(crate) trait OpenRGBConnection {
    async fn send_message<W: AsyncOpenRGBWriteExt>(
        writer: &mut W,
        command: Command,
        buffer: &[u8],
        device: Option<u32>,
    ) -> OpenRGBResult<()> {
        let header = PacketHeader {
            magic: 0,
            device: device.unwrap_or(0),
            command,
            length: buffer.len() as u32,
        };

        header.serialize(writer).await?;
        writer.write_all(buffer).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct PacketHeader {
    pub magic: u32,
    pub device: u32,
    pub command: Command,
    pub length: u32,
}

#[async_trait]
impl OpenRGBSendable for PacketHeader {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_u32(self.magic).await?;
        writer.write_u32(self.device).await?;
        writer.write_u32(self.command.clone() as u32).await?;
        writer.write_u32(self.length).await?;
        Ok(())
    }

    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let magic = reader.read_u32_le().await?;
        let device = reader.read_u32_le().await?;
        let command = Command::try_from(reader.read_u32_le().await?)?;
        let length = reader.read_u32_le().await?;

        Ok(Self {
            magic,
            device,
            command,
            length,
        })
    }
}
