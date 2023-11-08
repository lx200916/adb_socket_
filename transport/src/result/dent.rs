use crate::{transport::transport::AdbTransport, AdbTransportError};
use crate::utils::get_fail_message;
#[repr(C, packed)]
pub struct SyncDentV1 {
    mode: u32,
    size: u32,
    mtime: u32,
    pub namelen: u32,
    // name: [u8; namelen]
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncDent {
    pub mode: u32,
    pub size: u32,
    pub mtime: u32,
    pub name: String,
}
impl SyncDent {
    pub async fn from_le_bytes_transport(
        transport: &mut dyn AdbTransport,
    ) -> Result<Self, AdbTransportError> {
        let stat = String::from_utf8(transport.read_exact(4).await?).map_err(|err| {
            AdbTransportError::InvalidResponse("sync_dent".to_string(), Some(err.to_string()))
        })?;
        // println!("stat: {}", stat);
        match stat.as_str() {
            "DENT" => {}
            "FAIL" => {
                return Err(AdbTransportError::AdbError(
                     get_fail_message(transport).await?,
                ));
            }
            "DONE" =>{
                return Err(AdbTransportError::EOF);
            }

            _ => {
                return Err(AdbTransportError::InvalidResponse(
                    "sync_dent".to_string(),
                    Some(stat),
                ))
            }
            
        }
                let mut bytes = [0u8; std::mem::size_of::<SyncDentV1>()];
        transport.read_exact_(&mut bytes).await?;
        let mode = u32::from_le_bytes(bytes[..4].try_into()?);
        let size = u32::from_le_bytes(bytes[4..8].try_into()?);
        let mtime = u32::from_le_bytes(bytes[8..12].try_into()?);
        let namelen = u32::from_le_bytes(bytes[12..16].try_into()?);
        let mut name = vec![0u8; namelen as usize];
        transport.read_exact_(&mut name).await?;
        let name = String::from_utf8(name).map_err(|err| {
            AdbTransportError::InvalidResponse("sync_dent".to_string(), Some(err.to_string()))
        })?;



        Ok(Self {
            mode,
            size,
            mtime,
            name,
        })
    }
    // pub fn into_dent(self, name: String) -> SyncDent {
    //     SyncDent {
    //         mode: self.mode,
    //         size: self.size,
    //         mtime: self.mtime,
    //         name,
    //     }
    // }
}
