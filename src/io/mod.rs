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
pub trait AsyncOpenRGBWriteExt: AsyncWriteExt + Unpin + Send {}

#[async_trait]
pub trait AsyncOpenRGBReadExt: AsyncReadExt + Unpin + Send {}

impl<T> AsyncOpenRGBWriteExt for T where T: AsyncWriteExt + Unpin + Send {}

impl<T> AsyncOpenRGBReadExt for T where T: AsyncReadExt + Unpin + Send {}
