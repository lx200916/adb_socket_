use std::fmt::Display;

use anyhow::Result;
use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{multispace1, space1, digit1, one_of, space0},
    combinator::{opt, peek},
    error::ParseError,
    number::complete::{u64, u32},
    sequence::{preceded, tuple, terminated},
    IResult, branch::alt, Parser,
};
use nom::combinator::not;


pub enum Devices {
    Devices(Vec<Device>),
    Raw(String),
}
#[derive(Debug, PartialEq,)]
pub enum DeviceState {
    Connecting,   // Haven't received a response from the device yet.
    Authorizing,  // Authorizing with keys from ADB_VENDOR_KEYS.
    Unauthorized, // ADB_VENDOR_KEYS exhausted, fell back to user prompt.
    NoPerm,       // Insufficient permissions to communicate with the device.
    Detached,     // USB device that's detached from the adb server.
    Offline,
    Bootloader, // Device running fastboot OS (fastboot) or userspace fastboot (fastbootd).
    Device,     // Device running Android OS (adbd).
    Host,       // What a device sees from its end of a Transport (adb host).
    Recovery,   // Device with bootloader loaded but no ROM OS loaded (adbd).
    Sideload,   // Device running Android OS Sideload mode (minadbd sideload mode).
    Rescue,     // Device running Android OS Rescue mode (minadbd rescue mode).
}
impl DeviceState {
    fn parse_device_state(input: &str) -> IResult<&str, DeviceState> {
        let (input, state) = alt((
            tag("connecting"),
            tag("authorizing"),
            tag("unauthorized"),
            tag("noperm"),
            tag("detached"),
            tag("offline"),
            tag("bootloader"),
            tag("device"),
            tag("host"),
            tag("recovery"),
            tag("sideload"),
            tag("rescue"),
        ))(input)?;

        let state = match state {
            "connecting" => DeviceState::Connecting,
            "authorizing" => DeviceState::Authorizing,
            "unauthorized" => DeviceState::Unauthorized,
            "noperm" => DeviceState::NoPerm,
            "detached" => DeviceState::Detached,
            "offline" => DeviceState::Offline,
            "bootloader" => DeviceState::Bootloader,
            "device" => DeviceState::Device,
            "host" => DeviceState::Host,
            "recovery" => DeviceState::Recovery,
            "sideload" => DeviceState::Sideload,
            "rescue" => DeviceState::Rescue,
            _ => return Err(nom::Err::Error(ParseError::from_error_kind(input, nom::error::ErrorKind::Tag))),
        };

        Ok((input, state))
    }
}
impl ToString for DeviceState {
    fn to_string(&self) -> String {
        match self {
            DeviceState::Connecting => String::from("connecting"),
            DeviceState::Authorizing => String::from("authorizing"),
            DeviceState::Unauthorized => String::from("unauthorized"),
            DeviceState::NoPerm => String::from("noperm"),
            DeviceState::Detached => String::from("detached"),
            DeviceState::Offline => String::from("offline"),
            DeviceState::Bootloader => String::from("bootloader"),
            DeviceState::Device => String::from("device"),
            DeviceState::Host => String::from("host"),
            DeviceState::Recovery => String::from("recovery"),
            DeviceState::Sideload => String::from("sideload"),
            DeviceState::Rescue => String::from("rescue"),
        }
    }
    
}
#[derive(Debug)]
pub struct Device {
    pub serial: String,
    pub state: DeviceState,
    pub product: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub devpath: Option<String>,
    pub transport_id: Option<u64>,
}
impl TryFrom<&str> for Device {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self> {
        Self::nom_parse(value).map_err(|err| anyhow::Error::msg(err.to_string()))
    }
}
#[inline]
fn is_not_whitespace(c: char) -> bool {
    !c.is_whitespace()
}
impl Device {
    fn nom_parse(value: &str) -> Result<Device, nom::Err<nom::error::Error<&str>>> {
        let (value, serial) = take_while1(is_not_whitespace)(value)?;
        let (value, _) = space1(value)?;
        let (value, state) = DeviceState::parse_device_state(value)?;
        let (value, _) = space0(value)?;
        let (value, devpath) = opt(preceded(not(peek(tag("product:"))), take_while1(is_not_whitespace)))(value)?;
       
        let (value, _) = space0(value)?;
        let (_, (product, model, device, transport_id)) = tuple((
            opt(terminated( preceded(tag("product:"), take_while1(is_not_whitespace)),
            multispace1)),
            opt(terminated( preceded(tag("model:"), take_while1(is_not_whitespace)), multispace1)),
            opt(terminated( preceded(tag("device:"), take_while1(is_not_whitespace)), multispace1)),
            opt( preceded(tag("transport_id:"), digit1),),
        ))(value)?;
        Ok(Device {
            serial: serial.to_string(),
            state,
            product: product.map(|s| s.to_string()),
            model: model.map(|s| s.to_string()),
            device: device.map(|s| s.to_string()),
            devpath: devpath.map(|s| s.to_string()),
            transport_id: transport_id.map(|s| s.parse::<u64>().ok()).flatten(),
        })

    }
}
#[test]
fn test_devices_l(){
    let raw = r#"List of devices attached
emulator-5554          device product:sdk_phone_x86 model:Android_SDK_built_for_x86 device:generic_x86 transport_id:1
731d5853               device usb:34603008X product:houji model:23127PN0CC device:houji transport_id:5
"#;
let mut devices: Vec<Device> = vec![];
raw.split("\n").for_each(|line|{
    let line = line.trim();
    if line.is_empty(){
        return;
    }
    let device = Device::nom_parse(line).ok();
    println!("{:?}", device);
    if let Some(device) = device {
        devices.push(device);
    }
});
assert_eq!(devices.len(), 2);
assert_eq!(devices[0].serial, "emulator-5554");
assert_eq!(devices[0].state, DeviceState::Device);
assert_eq!(devices[0].product, Some("sdk_phone_x86".to_string()));
assert_eq!(devices[0].model, Some("Android_SDK_built_for_x86".to_string()));
assert_eq!(devices[0].device, Some("generic_x86".to_string()));
assert_eq!(devices[0].transport_id, Some(1));
assert_eq!(devices[1].serial, "731d5853");
assert_eq!(devices[1].state, DeviceState::Device);
assert_eq!(devices[1].product, Some("houji".to_string()));
assert_eq!(devices[1].model, Some("23127PN0CC".to_string()));
assert_eq!(devices[1].device, Some("houji".to_string()));
assert_eq!(devices[1].transport_id, Some(5));
}
#[test]
fn test_devices(){
    let raw = r#"
    731d5853	device
    emulator-5554	device"#;
    let mut devices: Vec<Device> = vec![];
    raw.split("\n").for_each(|line|{
        let line = line.trim();
        if line.is_empty(){
            return;
        }
        let device = Device::nom_parse(line).ok();
        println!("{:?}", device);
        if let Some(device) = device {
            devices.push(device);
        }
    });
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].serial, "731d5853");
    assert_eq!(devices[0].state, DeviceState::Device);
}