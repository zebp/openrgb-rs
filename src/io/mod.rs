use crate::OpenRGBResult;
use async_trait::async_trait;
use tokio::prelude::io::*;

// TODO: Rename this
#[async_trait]
pub trait OpenRGBSendable: Send {
    type Output: OpenRGBSendable;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()>;

    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output>;
}

#[async_trait]
pub trait AsyncOpenRGBWriteExt: AsyncWriteExt + Unpin + Send {
    async fn write_string(&mut self, value: &str) -> OpenRGBResult<()> {
        let bytes = value.as_bytes();

        self.write_u16_le((bytes.len() + 1) as u16).await?;
        self.write_all(bytes).await?;
        self.write_u8(0x00).await?; // OpenRGB's protocol has length prefixed AND null terminated strings

        Ok(())
    }
}

#[async_trait]
pub trait AsyncOpenRGBReadExt: AsyncReadExt + Unpin + Send {
    async fn read_string(&mut self) -> OpenRGBResult<String> {
        let len = self.read_u16_le().await? as usize;

        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer).await?;

        let buffer = &buffer[0..buffer.len() - 1];

        // TODO: Handle this error
        Ok(String::from_utf8(buffer.to_vec()).unwrap())
    }
}

impl<T> AsyncOpenRGBWriteExt for T where T: AsyncWriteExt + Unpin + Send {}

impl<T> AsyncOpenRGBReadExt for T where T: AsyncReadExt + Unpin + Send {}
