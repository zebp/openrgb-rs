use crate::OpenRGBError;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Command {
    SetClientName = 50,
    RequestControllerCount = 0,
    RequestControllerData = 1,
    UpdateLeds = 1050,
    UpdateZoneLeds = 1051,
    SetCustomMode = 1100,
}

impl TryFrom<u32> for Command {
    type Error = OpenRGBError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        Ok(match id {
            50 => Command::SetClientName,
            0 => Command::RequestControllerCount,
            1 => Command::RequestControllerData,
            1050 => Command::UpdateLeds,
            1051 => Command::UpdateZoneLeds,
            1100 => Command::SetCustomMode,
            _ => return Err(OpenRGBError::InvalidCommand(id)),
        })
    }
}
