use std::path::{self, Path};

use clap::Parser;
use transport::result::stat::FileType;
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Arguments {
    /// Serial number of the device to connect to
    #[clap(short, long)]
    pub serial: Option<String>,
    /// Unix socket to connect to server.
    #[clap(short, long, default_value = "/var/run/adb.socket")]
    pub socket: String,
    /// Use JSON output
    #[clap(long)]
    pub json: bool,
    #[clap(subcommand)]
    pub command: SubCommand,
}
#[derive(Parser, Debug)]
pub enum SubCommand {
    /// List connected devices
    Devices {
        /// List devices in long format
        #[clap(short, long)]
        long: bool,
    },
    /// Run a shell command on a device
    Shell {
        /// Command to run
        command: Vec<String>,
    },
    Push {
        filename: String,

        path: String,
    },
    Pull {
        path: String,
        filename: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let mut adb = transport::AdbTransports::new(args.socket, args.json)
        .await
        .unwrap();
    match args.command {
        SubCommand::Devices { long } => {
            let devices = if long {
                adb.devices_long().await.unwrap()
            } else {
                adb.devices().await.unwrap()
            };
            println!("List of devices attached");

            match devices {
                transport::result::device::Devices::Devices(devices) => {
                    devices.iter().for_each(|device| {
                        println!("{:?}", device);
                    });
                }
                transport::result::device::Devices::Raw(raw) => {
                    println!("{}", raw);
                }
            }
        }
        SubCommand::Shell { command } => {
            let command = command.join(" ");
            let resp = adb.shell(&args.serial, command).await.unwrap();
            println!("{}", String::from_utf8(resp).unwrap());
        }
        SubCommand::Push { filename, path } => {
            let path = Path::new(&path);

            // Check if path exists and is a folder?
            if !path.exists() {
                println!("Path does not exist");
                return;
            }
            if path.is_file() {
                let file = std::fs::File::open(path).unwrap();
                let mut reader = std::io::BufReader::new(file);
                 adb.push(args.serial, &mut reader, filename).await.unwrap();
            }else if path.is_dir() {
                //check if `filename` is a dir?
                
                if adb.sync_stat(filename.clone(), args.serial.clone()).await.unwrap().get_file_type() != FileType::Directory {
                    println!("can not push folder to a file:{}", filename);
                    return;
                }
                let filename = Path::new(&filename);
                // Walk the directory recursively and push all files, mkdir all folders
                let walker = walkdir::WalkDir::new(path);
                let base = path;
                for entry in walker
                    .into_iter()
                    .filter_map(|e| e.ok()){
                       
                        let path = entry.path();
                        if path.is_file() && !path.is_symlink() {
                            let file = std::fs::File::open(path).unwrap();
                            let mut reader = std::io::BufReader::new(file);
                            // get the remote path of the file, relative to the base path,then concat with  remote filename
                            let filename_ = path.strip_prefix(base).unwrap();
                            let filename_ = filename.join(filename_);
                            adb.push(args.serial.clone(), &mut reader, filename_.to_str().unwrap()).await.unwrap();
                            println!("push file:{} to  {}", filename.to_str().unwrap(),filename_.to_str().unwrap())                            
                        }else if path.is_dir() {
                            let filename_ = path.strip_prefix(path).unwrap();
                            let filename_ = filename.join(filename_);
                            adb.mkdir(args.serial.clone(), filename.to_str().unwrap()).await.unwrap();
                            println!("mkdir {}", filename_.to_str().unwrap());
                        }
                    }


            }
        }
        SubCommand::Pull { path, filename } => {
            let stat_info = adb.sync_stat(path.clone(), args.serial.clone()).await.unwrap();
            if stat_info.get_file_type() == FileType::Directory {
                println!("Pulling A Directory is not supported yet");
                return;
            }
            let mut file = std::fs::File::create(filename).unwrap();
            adb.pull(args.serial, path, &mut file).await.unwrap();
        }
    }
}
