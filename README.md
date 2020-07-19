# openrgb
An implementation of the [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB) SDK protocol in Rust.

## Note

### Server Implementation
The implementation of an OpenRGB SDK server is still W.I.P.

### Tokio
This project uses the [tokio](https://github.com/tokio-rs/tokio/) runtime but is required to use the Git version until the next release due to the addition of `Little Endian` methods to the async read/write extension traits.

## Example

```rust
use openrgb::*;

const RED: OpenRGBColor = (0xFF, 0x00, 0x00);

#[tokio::main]
async fn main() -> OpenRGBResult<()> {
    let mut client = OpenRGBClient::connect("0.0.0.0:6742", "Example").await?;
    let device_count = client.get_device_count().await?;

    for device_id in 0..device_count {
        let device = client.get_device(device_id).await?;

        client.set_custom_mode(device_id).await?;
        client.update_leds(device_id, &vec![RED; device.colors.len()]).await?;
    }

    Ok(())
}
```
