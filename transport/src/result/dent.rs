use crate::AdbTransportError;
#[repr(C, packed)]
pub struct SyncDentV1 {
    id: u32,
    mode: u32,
    size: u32,
    mtime: u32,
   pub namelen: u32,
    // name: [u8; namelen] 
}
pub struct SyncDent {
    pub mode: u32,
    pub size: u32,
    pub mtime: u32,
    pub name: String,
}
impl SyncDentV1 {
    pub fn from_le_bytes(bytes: [u8; std::mem::size_of::<SyncDentV1>()]) -> Result<Self, AdbTransportError> {
        let id = u32::from_le_bytes(bytes[0..4].try_into()?);
        let stat = String::from_utf8(bytes[4..8].to_vec()).map_err(|_| AdbTransportError::InvalidResponse)?;
        if stat != "DENT" {
            return Err(AdbTransportError::EOF);
        }
        let mode = u32::from_le_bytes(bytes[8..12].try_into()?);
        let size = u32::from_le_bytes(bytes[12..16].try_into()?);
        let mtime = u32::from_le_bytes(bytes[16..20].try_into()?);
        let namelen = u32::from_le_bytes(bytes[20..24].try_into()?);
        Ok(Self {
            id,
            mode,
            size,
            mtime,
            namelen,
        })

    }
    pub fn into_dent(self, name: String) -> SyncDent {
        SyncDent {
            mode: self.mode,
            size: self.size,
            mtime: self.mtime,
            name,
        }
    }
    
}