use std::time::Duration;
use serial::prelude::*;
use anyhow::{Result, bail};
use std::fs::File;
use serde::{Serialize, Deserialize};
use csv;

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud9600,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Setting {
    power_level : u16,
    high_pressure : u16,
    mid_pressure : u16,
    low_pressure : u16,
    high_pulse : u16,
    mid_pulse : u16,
    low_pulse : u16,
    high_slope : u16,
    low_slope : u16,
    volts : u16
}

impl Setting {
    fn new( power_level : u16,
            high_pressure : u16,
            mid_pressure : u16,
            low_pressure : u16,
            high_pulse : u16,
            mid_pulse : u16,
            low_pulse : u16,
            high_slope : u16,
            low_slope : u16,
            volts : u16) -> Self {
        Setting {
            power_level,
            high_pressure,
            mid_pressure,
            low_pressure,
            high_pulse,
            mid_pulse,
            low_pulse,
            high_slope,
            low_slope,
            volts
        }
    }
}

pub struct Gcu {
    port: Box<dyn SerialPort>,
    settings : Vec<Setting>,
}

impl Gcu {
    pub fn new<P: 'static + SerialPort>(mut port: P) -> Result<Self> {
        port.configure(&SETTINGS)?;
        port.set_timeout(Duration::from_secs(1))?;
        Ok(Gcu { port: Box::new(port), settings : Vec::new() })
    }

    /* Read bytes from the uart until we encounter a '/r' */
    fn read_string(&mut self) -> Result<Vec<u8>> {
        let mut c: Vec<u8> = vec![0; 1];
        let mut result: Vec<u8> = Vec::new();
        loop {
            self.port.read(&mut c)?;
            match c[0] {
                0x0D => break,
                _ => result.push(c[0]),
            }
        }
        Ok(result)
    }

    fn fetch_response(&mut self) -> Result<Option<Vec<u8>>> {
        let mut result : Option<Vec<u8>> = None;
        loop{
            let v = self.read_string()?;
            let s = std::str::from_utf8(&v)?;
            match s {
                "OK" => {break;},
                "Error" => {break;},
                _ => {result = Some(v)}
            }
        }
        Ok(result)
    }

    /* Send the 7 byte command, consume the echo from 1 wire operation */
    fn send_cmd(&mut self, cmd : &str) -> Result<()> {
        let mut echo: Vec<u8> = vec![0; 1];
        let millis = std::time::Duration::from_millis(200);
        std::thread::sleep(millis);
        self.port.write(cmd.as_bytes())?;
        for _ in 0..cmd.len() {
            self.port.read(&mut echo)?;
        }
        Ok(())
    }

    pub fn connect(&mut self) -> Result<()> {
        self.port.set_dtr(true)?;
        let millis = std::time::Duration::from_millis(500);
        std::thread::sleep(millis);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.send_cmd(&"Q      ")?;
        self.port.set_dtr(false)?;
        Ok(())
    }

    fn write_word(&mut self, address : u8, data : u16) -> Result<()> {
        let scmd = format!("W{:02}{:04}", address, data);
        self.send_cmd(&scmd)?;
        self.fetch_response()?;
        return Ok(());
    }

    fn read_u16(&mut self, cmd : &str) -> Result<u16> {
        self.send_cmd(cmd)?;
        if let Some(response) = self.fetch_response()? {
            let resp = std::str::from_utf8(&response)?.parse::<u16>()?;
            return Ok(resp);
        }
        bail!("Error received");
    }

    fn read_word(&mut self, address : u8) -> Result<u16> {
        let cmd = format!("R{:02}0000", address);
        self.read_u16(&cmd)
    }

    pub fn pressure(&mut self) -> Result<u16> {
        self.read_u16(&"P      ")
    }

    pub fn pulse_duration(&mut self) -> Result<u16> {
        self.read_u16(&"A      ")
    }

    pub fn version(&mut self) -> Result<String> {
        self.send_cmd(&"V      ")?;
        if let Some(response) = self.fetch_response()? {
            let sresp = std::str::from_utf8(&response)?.to_owned();
            return Ok(sresp);
        }
        bail!("Error received");
    }


    pub fn read_settings(&mut self) -> Result<()> {
        for i in 1..4 {
            let high_pressure : u16 = self.read_word(i * 20)?;
            let mid_pressure : u16 = self.read_word(2 + i * 20)?;
            let low_pressure : u16 = self.read_word(4 + i * 20)?;
            let high_pulse : u16 = self.read_word(6 + i * 20)?;
            let mid_pulse : u16 = self.read_word(8 + i * 20)?;
            let low_pulse : u16 = self.read_word(10 + i * 20)?;
            let high_slope : u16 = self.read_word(12 + i * 20)?;
            let low_slope : u16 = self.read_word(14 + i * 20)?;
            let volts : u16 = self.read_word(16 + i * 20)?;
            let setting = Setting::new(     i as u16,
                                            high_pressure,
                                            mid_pressure,
                                            low_pressure,
                                            high_pulse,
                                            mid_pulse,
                                            low_pulse,
                                            high_slope,
                                            low_slope,
                                            volts);
            self.settings.push(setting);
        }
        Ok(())
    }

    pub fn write_settings(&mut self, idx : usize) -> Result<()> {
        self.write_word(20 * self.settings[idx].power_level as u8, self.settings[idx].high_pressure)?;
        self.write_word(2 + 20 * self.settings[idx].power_level as u8, self.settings[idx].mid_pressure)?;
        self.write_word(4 + 20 * self.settings[idx].power_level as u8, self.settings[idx].low_pressure)?;
        self.write_word(6 + 20 * self.settings[idx].power_level as u8, self.settings[idx].high_pulse)?;
        self.write_word(8 + 20 * self.settings[idx].power_level as u8, self.settings[idx].mid_pulse)?;
        self.write_word(10 + 20 * self.settings[idx].power_level as u8, self.settings[idx].low_pulse)?;
        self.write_word(12 + 20 * self.settings[idx].power_level as u8, self.settings[idx].high_slope)?;
        self.write_word(14 + 20 * self.settings[idx].power_level as u8, self.settings[idx].low_slope)?;
        self.write_word(16 + 20 * self.settings[idx].power_level as u8, self.settings[idx].volts)?;
        Ok(())
    }

    /* Write the 3 power settings as a .csv */
    pub fn serialize_settings(&mut self, filename : &str) -> Result<()> {
        let file = File::create(filename)?;
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .flexible(true)
            .double_quote(false)
            .from_writer(file);
        for s in &self.settings {
            wtr.serialize(s)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /* Read the power settings from a .csv */
    pub fn deserialize_settings(&mut self, filename : &str) -> Result<()> {
        let file = File::open(filename)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .double_quote(false)
            .from_reader(file);
        for result in rdr.deserialize() {
            let setting: Setting = result?;
            self.settings.push(setting);
        }
        Ok(())
    }
}
