use crate::{AdbTransports, result::device::Devices, AdbTransportError};
use anyhow::Result;
use crate::result::device::Device;
impl AdbTransports {
    async fn devices(&mut self)->Result<Devices>{
        let resp = self.transports.send_command(crate::AdbCommand::Devices, true).await?;
        if self.json{
            let resp = String::from_utf8(resp).map_err(|err|AdbTransportError::AdbError(err.to_string()))?;
            let mut devices: Vec<Device> = Vec::new();
            resp.split("\n").for_each(|line|{
                if let Ok(device) = Device::try_from(line){
                    devices.push(device);
                }
            });
             Ok(Devices::Devices(devices))
        }else {
            return Ok(Devices::Raw(String::from_utf8(resp).map_err(|err|AdbTransportError::AdbError(err.to_string()))?));
        }
    }
    async fn devices_long(&mut self)->Result<Devices>{
        let resp = self.transports.send_command(crate::AdbCommand::DevicesLong, true).await?;
        if self.json{
                    let resp = String::from_utf8(resp).map_err(|err|AdbTransportError::AdbError(err.to_string()))?;
            let mut devices: Vec<Device> = Vec::new();
            resp.split("\n").for_each(|line|{
                if let Ok(device) = Device::try_from(line){
                    devices.push(device);
                }
            });
             Ok(Devices::Devices(devices))
        }else {
            return Ok(Devices::Raw(String::from_utf8(resp).map_err(|err|AdbTransportError::AdbError(err.to_string()))?));
        }
    }
    async fn track_device(&mut self,callback: impl Fn(Device))->Result<()>{
        
        self.transports.send_command(crate::AdbCommand::TrackDevices, true).await?;
        loop {
            let length = self.transports.get_length().await?;
            let message = self.transports.read_exact(length).await?;
            let message = String::from_utf8(message).ok();
            if let Some(message) = message {
                if let Ok(device) = Device::try_from(message.as_str()){
                    callback(device);
                }
            }
        }
    }
}