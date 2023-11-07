use crate::transport::transport::AdbTransport;
use anyhow::Ok;
use transport::unix_stream_transport::UnixStreamTransport;
mod commands;
pub mod result;
mod transport;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum AdbTransportError {
    #[error("IO Error {0:?}")]
    IoError(std::io::Error),
    #[error("Adb Transport Error: {0:?}")]
    AdbError(String),
    #[error("Invalid Response @{0} :{1:?}")]
    InvalidResponse(String, Option<String>),
    #[error("Response Conversion Error: {0:?}")]
    ConversionError(String),
    #[error("EOF")]
    EOF,
}
use std::array::TryFromSliceError;

impl From<TryFromSliceError> for AdbTransportError {
    fn from(err: TryFromSliceError) -> Self {
        AdbTransportError::InvalidResponse("TryFromSlice".to_string(), Some(err.to_string()))
    }
}
// https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/SERVICES.TXT
pub enum AdbCommand {
    Version,
    Devices,
    ShellExec(String),
    DevicesLong,
    Sync,
    TransportAny,
    TransportSerial(String),
    TrackDevices,
}

impl ToString for AdbCommand {
    fn to_string(&self) -> String {
        match self {
            AdbCommand::Version => String::from("host:version"),
            AdbCommand::Devices => String::from("host:devices"),
            AdbCommand::ShellExec(cmd) => format!("shell,raw:{}", cmd),
            AdbCommand::DevicesLong => String::from("host:devices-l"),
            AdbCommand::Sync => String::from("sync:"),
            AdbCommand::TransportAny => String::from("host:transport-any"),
            AdbCommand::TransportSerial(serial) => format!("host:transport:{}", serial),
            AdbCommand::TrackDevices => String::from("host:track-devices"),
        }
    }
}
// https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/SYNC.TXT
// https://cs.android.com/android/platform/superproject/main/+/main:packages/modules/adb/file_sync_protocol.h
pub enum AdbSyncModeCommand {
    Send,
    Recv,
    List,
    Stat,
}
impl ToString for AdbSyncModeCommand {
    fn to_string(&self) -> String {
        match self {
            AdbSyncModeCommand::Send => String::from("SEND"),
            AdbSyncModeCommand::Recv => String::from("RECV"),
            AdbSyncModeCommand::List => String::from("LIST"),
            AdbSyncModeCommand::Stat => String::from("STAT"),
        }
    }
}
impl Into<Vec<u8>> for AdbSyncModeCommand {
    fn into(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
enum AdbRespStatus {
    Okay,
    Fail(String),
}
// impl TryFrom<Vec<u8>> for AdbRespStatus {
//     type Error = anyhow::Error;

//     fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
//         let status = value.get(0..4).ok_or(AdbTransportError::InvalidResponse)?;
//         let status = std::str::from_utf8(status)?;
//         match status {
//             "OKAY" => Ok(AdbRespStatus::Okay),
//             "FAIL" => {
//                 let length = value.get(4..8).ok_or(AdbTransportError::InvalidResponse)?;
//                 let length = std::str::from_utf8(length)?;
//                 let length = usize::from_str_radix(length, 16)?;
//                 let message = value
//                     .get(8..length)
//                     .ok_or(AdbTransportError::InvalidResponse)?;
//                 let message = std::str::from_utf8(message)?;
//                 Ok(AdbRespStatus::Fail(message.to_string()))
//             }
//             _ => Err(AdbTransportError::InvalidResponse.into()),
//         }
//     }
// }
impl From<[u8; 4]> for AdbRespStatus {
    fn from(value: [u8; 4]) -> Self {
        let status = String::from_utf8(value.to_vec())
            .unwrap()
            .to_ascii_lowercase();
        println!("{:?}", status);

        match status.as_str() {
            "okay" => AdbRespStatus::Okay,
            "fail" => AdbRespStatus::Fail(String::from("")),
            _ => AdbRespStatus::Fail(String::from("")),
        }
    }
}
pub struct AdbTransports {
    serial_set: bool,
    transports: Box<dyn AdbTransport>,
    json: bool,
}
impl AdbTransports {
    pub async fn new(sockt: String, json: bool) -> anyhow::Result<Self> {
        if !sockt.starts_with("/") {
            anyhow::bail!("only Unix Socket is supported");
        }
        let transports = Box::new(UnixStreamTransport::new(sockt).await?);
        Ok(Self {
            transports,
            json,
            serial_set: false,
        })
    }
    pub async fn may_set_serial<S: ToString>(&mut self, serial: Option<S>) -> anyhow::Result<()> {
        if !self.serial_set {
            let transport_ = match serial {
                Some(serial) => AdbCommand::TransportSerial(serial.to_string()),
                None => AdbCommand::TransportAny,
            };
            self.transports.send_command(transport_, false).await?;
            self.serial_set = true;
        }
        Ok(())
    }
    pub async fn new_connection(&mut self)->anyhow::Result<()>{
        self.transports.reconnect().await?;
        self.serial_set = false;
        Ok(())
    }
}
