use crate::{AdbTransportError, utils::get_fail_message, transport::transport::AdbTransport};
const S_IFMT: u32 = 0o170000;
const S_IFSOCK: u32 = 0o140000;
const S_IFLNK: u32 = 0o120000;
const S_IFREG: u32 = 0o100000;
const S_IFBLK: u32 = 0o060000;
const S_IFDIR: u32 = 0o040000;
const S_IFCHR: u32 = 0o020000;
const S_IFIFO: u32 = 0o010000;
const S_ISUID: u32 = 0o004000;
const S_ISGID: u32 = 0o002000;
const S_ISVTX: u32 = 0o001000;
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatInfo {
    mode: u32,
    size: u32,
    mtime: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Link,
    BlockDevice,
    CharDevice,
    Socket,
    Fifo,
    Other,
}

impl StatInfo {
    //    pub  fn from_le_bytes(bytes: [u8; std::mem::size_of::<StatInfo>()]) -> Result<Self,AdbTransportError> {
    //         let id = u32::from_le_bytes(bytes[0..4].try_into()?);
    //         let stat = String::from_utf8(bytes[0..4].to_vec()).map_err(|err| AdbTransportError::InvalidResponse("sync_stat".to_string(),Some(err.to_string())))?;
    //         match stat.as_str() {
    //             "STAT" => {},
    //             "FAIL" =>{
    //                 let msg = String::from_utf8(bytes[4..].to_vec()).map_err(|err| AdbTransportError::InvalidResponse("sync_stat".to_string(),Some(err.to_string())))?;
    //                 return Err(AdbTransportError::InvalidResponse("sync_stat".to_string(),Some(msg)));
    //             },

    //             _=>return Err(AdbTransportError::InvalidResponse("sync_stat".to_string(),Some(stat)))

    //         }
    //         let mode = u32::from_le_bytes(bytes[4..8].try_into()?);
    //         let size = u32::from_le_bytes(bytes[8..12].try_into()?);
    //         let mtime = u32::from_le_bytes(bytes[12..16].try_into()?);
    //         Ok(Self {
    //             id,
    //             mode,
    //             size,
    //             mtime,
    //         })
    //     }
    pub async fn from_le_bytes_transport(
        transport: &mut dyn AdbTransport,
    ) -> Result<Self, AdbTransportError> {
        let stat = String::from_utf8(transport.read_exact(4).await?).map_err(|err| {
            AdbTransportError::InvalidResponse("sync_stat".to_string(), Some(err.to_string()))
        })?;
        match stat.as_str() {
            "STAT" => {}
            "FAIL" => {
                return Err(AdbTransportError::AdbError(
                    get_fail_message(transport).await?,
                ));
            }
            "DONE" => {
                return Err(AdbTransportError::EOF);
            }

            _ => {
                return Err(AdbTransportError::InvalidResponse(
                    "sync_stat".to_string(),
                    Some(stat),
                ))
            }
        }
        let mut bytes = [0u8; std::mem::size_of::<StatInfo>()];
        transport.read_exact_(&mut bytes).await?;
        let mode = u32::from_le_bytes(bytes[..4].try_into()?);
        let size = u32::from_le_bytes(bytes[4..8].try_into()?);
        let mtime = u32::from_le_bytes(bytes[8..12].try_into()?);
        Ok(Self { mode, size, mtime })
        
        
    }
    pub fn get_file_type(&self) -> FileType {
        match self.mode & S_IFMT {
            S_IFSOCK => FileType::Socket,
            S_IFLNK => FileType::Link,
            S_IFREG => FileType::File,
            S_IFBLK => FileType::BlockDevice,
            S_IFDIR => FileType::Directory,
            S_IFCHR => FileType::CharDevice,
            S_IFIFO => FileType::Fifo,
            _ => FileType::Other,
        }
    }
}
