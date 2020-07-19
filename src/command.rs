use crate::OpenRGBError;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Command {
    SetClientName = 50,
    RequestControllerCount = 0,
    RequestControllerData = 1,
    ResizeZone = 1000,
    UpdateLeds = 1050,
    UpdateZoneLeds = 1051,
    UpdateSingleLed = 1052,
    SetCustomMode = 1100,
    UpdateMode = 1101,
}

impl TryFrom<u32> for Command {
    type Error = OpenRGBError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        Ok(match id {
            50 => Command::SetClientName,
            0 => Command::RequestControllerCount,
            1 => Command::RequestControllerData,
            1000 => Command::ResizeZone,
            1050 => Command::UpdateLeds,
            1051 => Command::UpdateZoneLeds,
            1052 => Command::UpdateSingleLed,
            1100 => Command::SetCustomMode,
            1101 => Command::UpdateMode,
            _ => return Err(OpenRGBError::InvalidCommand(id)),
        })
    }
}
