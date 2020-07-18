use crate::{
    io::{AsyncOpenRGBReadExt, AsyncOpenRGBWriteExt, OpenRGBSendable},
    OpenRGBResult,
};
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

pub type OpenRGBColor = u32;
pub type OpenRGBZoneType = u32;

#[derive(Debug, Clone)]
pub struct OpenRGBMode {
    pub name: String,
    pub value: i32,
    pub flags: u32,
    pub speed_min: u32,
    pub speed_max: u32,
    pub colors_min: u32,
    pub colors_max: u32,
    pub speed: u32,
    pub direction: u32,
    pub color_mode: u32,
    pub colors: Vec<OpenRGBColor>,
}

#[derive(Debug, Clone)]
pub struct OpenRGBLed {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct OpenRGBMatrixMap {
    pub height: u32,
    pub width: u32,
    pub map: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct OpenRGBZone {
    pub name: String,
    // TODO: Refactor this to an enum.
    pub zone_type: OpenRGBZoneType,
    pub leds_count: u32,
    pub leds_min: u32,
    pub leds_max: u32,
    pub matrix_map: Option<OpenRGBMatrixMap>,
}

#[derive(Debug, Clone)]
pub struct OpenRGBDevice {
    pub name: String,
    pub description: String,
    pub version: String,
    pub serial: String,
    pub location: String,
    pub leds: Vec<OpenRGBLed>,
    pub zones: Vec<OpenRGBZone>,
    pub modes: Vec<OpenRGBMode>,
    pub colors: Vec<OpenRGBColor>,
    pub device_type: u32,
    pub active_mode: i32,
}

#[async_trait]
impl OpenRGBSendable for OpenRGBDevice {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        let mut buffer = Vec::new();

        buffer.write_u32_le(self.device_type).await?;
        buffer.write_string(&self.name).await?;
        buffer.write_string(&self.description).await?;
        buffer.write_string(&self.version).await?;
        buffer.write_string(&self.serial).await?;
        buffer.write_string(&self.location).await?;

        buffer.write_u16_le(self.modes.len() as u16).await?;
        for mode in &self.modes {
            mode.serialize(&mut buffer).await?;
        }

        buffer.write_u16_le(self.zones.len() as u16).await?;
        for zone in &self.zones {
            zone.serialize(&mut buffer).await?;
        }

        buffer.write_u16_le(self.leds.len() as u16).await?;
        for led in &self.leds {
            led.serialize(&mut buffer).await?;
        }

        buffer.write_u16_le(self.colors.len() as u16).await?;
        for color in &self.colors {
            buffer.write_u32_le(*color).await?;
        }

        writer.write_u32_le(buffer.len() as u32).await?;
        writer.write_all(&buffer).await?;

        Ok(())
    }

    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let _ = reader.read_u32_le().await?;
        let device_type = reader.read_u32_le().await?;
        let name = reader.read_string().await?;
        let description = reader.read_string().await?;
        let version = reader.read_string().await?;
        let serial = reader.read_string().await?;
        let location = reader.read_string().await?;

        let mode_count = reader.read_u16_le().await? as usize;
        let active_mode = reader.read_i32_le().await?;

        let mut modes = Vec::with_capacity(mode_count);

        for _ in 0..mode_count {
            let mode = OpenRGBMode::deserialize(reader).await?;
            modes.push(mode);
        }

        let zone_count = reader.read_u16_le().await? as usize;
        let mut zones = Vec::with_capacity(zone_count);

        for _ in 0..zone_count {
            let zone = OpenRGBZone::deserialize(reader).await?;
            zones.push(zone);
        }

        let led_count = reader.read_u16_le().await? as usize;
        let mut leds = Vec::with_capacity(led_count);

        for _ in 0..led_count {
            let led = OpenRGBLed::deserialize(reader).await?;
            leds.push(led);
        }

        let color_count = reader.read_u16_le().await? as usize;
        let mut colors: Vec<OpenRGBColor> = Vec::with_capacity(color_count);

        for _ in 0..color_count {
            let color = reader.read_u32_le().await?;
            colors.push(color);
        }

        Ok(Self {
            name,
            description,
            version,
            serial,
            location,
            leds,
            zones,
            modes,
            colors,
            device_type,
            active_mode,
        })
    }
}

#[async_trait]
impl OpenRGBSendable for OpenRGBMode {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_string(&self.name).await?;
        writer.write_i32_le(self.value).await?;
        writer.write_u32_le(self.flags).await?;
        writer.write_u32_le(self.speed_min).await?;
        writer.write_u32_le(self.speed_max).await?;
        writer.write_u32_le(self.colors_min).await?;
        writer.write_u32_le(self.colors_max).await?;
        writer.write_u32_le(self.speed).await?;
        writer.write_u32_le(self.direction).await?;
        writer.write_u32_le(self.color_mode).await?;

        writer.write_u16_le(self.colors.len() as u16).await?;

        for color in &self.colors {
            writer.write_u32_le(*color).await?;
        }

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let name = reader.read_string().await?;
        let value = reader.read_i32_le().await?;
        let flags = reader.read_u32_le().await?;
        let speed_min = reader.read_u32_le().await?;
        let speed_max = reader.read_u32_le().await?;
        let colors_min = reader.read_u32_le().await?;
        let colors_max = reader.read_u32_le().await?;
        let speed = reader.read_u32_le().await?;
        let direction = reader.read_u32_le().await?;
        let color_mode = reader.read_u32_le().await?;

        let color_count = reader.read_u16_le().await? as usize;
        let mut colors = Vec::with_capacity(color_count);

        for _ in 0..color_count {
            let color = reader.read_u32_le().await?;
            colors.push(color);
        }

        Ok(Self {
            name,
            value,
            flags,
            speed_min,
            speed_max,
            colors_min,
            colors_max,
            speed,
            direction,
            color_mode,
            colors,
        })
    }
}

#[async_trait]
impl OpenRGBSendable for OpenRGBZone {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_string(&self.name).await?;
        writer.write_u32_le(self.zone_type).await?;
        writer.write_u32_le(self.leds_min).await?;
        writer.write_u32_le(self.leds_max).await?;
        writer.write_u32_le(self.leds_count).await?;

        let mut matrix_buffer = Vec::new();
        if let Some(matrix_map) = &self.matrix_map {
            matrix_map.serialize(&mut matrix_buffer).await?
        }

        writer.write_u16_le(matrix_buffer.len() as u16).await?;

        if matrix_buffer.len() > 0 {
            writer.write_all(&matrix_buffer).await?;
        }

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let name = reader.read_string().await?;
        let zone_type = reader.read_u32_le().await?;
        let leds_min = reader.read_u32_le().await?;
        let leds_max = reader.read_u32_le().await?;
        let leds_count = reader.read_u32_le().await?;

        let matrix_size = reader.read_u16_le().await? as usize;
        let matrix_map = if matrix_size > 0 {
            let matrix_map = OpenRGBMatrixMap::deserialize(reader).await?;
            Some(matrix_map)
        } else {
            None
        };

        Ok(Self {
            name,
            zone_type,
            leds_min,
            leds_max,
            leds_count,
            matrix_map,
        })
    }
}

#[async_trait]
impl OpenRGBSendable for OpenRGBMatrixMap {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_u32_le(self.height).await?;
        writer.write_u32_le(self.width).await?;

        for color in &self.map {
            writer.write_u32_le(*color).await?;
        }

        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let height = reader.read_u32_le().await?;
        let width = reader.read_u32_le().await?;
        let size = (width * height) as usize;

        let mut map = Vec::with_capacity(size);
        for _ in 0..size {
            let color = reader.read_u32_le().await?;
            map.push(color);
        }

        Ok(Self { height, width, map })
    }
}

#[async_trait]
impl OpenRGBSendable for OpenRGBLed {
    type Output = Self;

    async fn serialize<W: AsyncOpenRGBWriteExt + Send + Unpin>(
        &self,
        writer: &mut W,
    ) -> OpenRGBResult<()> {
        writer.write_string(&self.name).await?;
        writer.write_u32_le(self.value).await?;
        Ok(())
    }
    async fn deserialize<R: AsyncOpenRGBReadExt>(reader: &mut R) -> OpenRGBResult<Self::Output> {
        let name = reader.read_string().await?;
        let value = reader.read_u32_le().await?;

        Ok(Self { name, value })
    }
}
