# openrgb
An implementation of the OpenRGB SDK protocol in Rust.

## Note
The implementation of an OpenRGB SDK server is still W.I.P.

## Example

```rust
use openrgb::*;

#[tokio::main]
async fn main() -> OpenRGBResult<()> {
    let mut client = OpenRGBClient::connect("0.0.0.0:6742", "Example").await?;
    let device_count = client.get_device_count().await?;

    for device_id in 0..device_count {
        let device = client.get_device(device_id).await?;

        client.set_custom_mode(device_id).await?;
        client.update_leds(device_id, &vec![(0xFF, 0x0, 0x0); device.colors.len()]).await?;
    }

    Ok(())
}
```