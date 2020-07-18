use super::packet::*;
use crate::{
    command::Command,
    io::{AsyncOpenRGBReadExt, AsyncOpenRGBWriteExt, OpenRGBSendable},
    OpenRGBResult,
};
use async_trait::async_trait;
use std::{convert::TryFrom, io::Cursor};
use tokio::io::AsyncReadExt;

const MAGIC: u32 = 1111970383;

#[async_trait]
pub(crate) trait OpenRGBConnection {
    async fn send_command<W: AsyncOpenRGBWriteExt>(
        writer: &mut W,
        command: Command,
        device: Option<u32>,
    ) -> OpenRGBResult<()> {
        let header = PacketHeader {
            magic: MAGIC,
            device: device.unwrap_or(0),
            command,
            length: 0,
        };

        header.serialize(writer).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn send_packet<W: AsyncOpenRGBWriteExt, P: OpenRGBPacket>(
        writer: &mut W,
        packet: P,
        device: Option<u32>,
    ) -> OpenRGBResult<()> {
        let command = packet.command();
        let mut buffer = Vec::new();
        packet.serialize(&mut buffer).await?;

        let header = PacketHeader {
            magic: MAGIC,
            device: device.unwrap_or(0),
            command,
            length: buffer.len() as u32,
        };

        header.serialize(writer).await?;
        writer.write_all(&buffer).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn read_command<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Command> {
        let header = PacketHeader::deserialize(reader).await?;
        Ok(header.command)
    }

    async fn read_packet<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<OpenRGBPackets> {
        let header = PacketHeader::deserialize(reader).await?;
        let mut buffer = vec![0u8; header.length as usize];
        reader.read_exact(&mut buffer).await?;

        let mut buffer = Cursor::new(buffer);

        let packet = match header.command {
            Command::SetClientName => {
                OpenRGBPackets::SetClientName(SetClientNamePacket::deserialize(&mut buffer).await?)
            }
            Command::RequestControllerCount => OpenRGBPackets::RequestControllerCount(
                RequestControllerCountPacket::deserialize(&mut buffer).await?,
            ),
            Command::RequestControllerData => OpenRGBPackets::RequestControllerData(
                RequestControllerDataPacket::deserialize(&mut buffer).await?,
            ),
            _ => todo!(),
        };

        Ok(packet)
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
        writer.write_u32_le(self.magic).await?;
        writer.write_u32_le(self.device).await?;
        writer.write_u32_le(self.command.clone() as u32).await?;
        writer.write_u32_le(self.length).await?;
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
