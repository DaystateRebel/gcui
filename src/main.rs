use anyhow::Result;
use clap::{arg, Command};
mod gcu;
use gcu::*;

fn main() -> Result<()> {
    let matches = Command::new("gcui")
        .arg(arg!(-p --port ... "The serial port to use")
            .required(true)
            .takes_value(true)
        )
        .arg(arg!(-w --write ... "Write the power settings to GCU")
            .requires("filename")
            .conflicts_with_all(&["read", "rwversion"])
        )
        .arg(arg!(-r --read ... "Read the power settings from GCU")
            .requires("filename")
            .conflicts_with_all(&["write", "rwversion"])
        )
        .arg(arg!(-f --filename ... "The filename to read/write")
            .takes_value(true)
        )
        .arg(arg!(-e --rwversion ... "Read the version string from GCU")
        )
        .arg(arg!(-P --pressure ... "Read the current pressure from GCU")
        )
        .arg(arg!(-A --pulse ... "Read the current pulse duration from GCU")
        )
        .get_matches();

    let port_name = matches.value_of("port").unwrap();
    let port = serial::open(port_name)?;
    let mut gcu = Gcu::new(port)?;
    gcu.connect()?;

    if matches.is_present("write") {
        let filename = matches.value_of("filename").unwrap();
        gcu.deserialize_settings(&filename)?;
        for i in 0..3 {
            gcu.write_settings(i)?;
        }
    }
    if matches.is_present("read") {
        let filename = matches.value_of("filename").unwrap();
        gcu.read_settings()?;
        gcu.serialize_settings(&filename)?;
    }
    if matches.is_present("rwversion") {
        print!("Version : {}\n\r",gcu.version()?);
    }
    if matches.is_present("pressure") {
        print!("Pressure : {}\n\r",gcu.pressure()?);
    }
    if matches.is_present("pulse") {
        print!("Pulse Duration : {}\n\r",gcu.pulse_duration()?);
    }
   
    gcu.disconnect()?;
    Ok(())
}

