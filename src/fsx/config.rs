//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::result;

use log::LogLevelFilter;
use rustc_serialize::*;
use toml;

pub enum Error {
	CannotParse,
	CannotDecode,
}

pub type Result<T> = result::Result<T, Error>;

pub struct LoggingSettings {
    log_level: LogLevelFilter,
}

impl Decodable for LoggingSettings {
    fn decode<D: Decoder>(d: &mut D) -> result::Result<Self, D::Error> {
        let mut result = LoggingSettings::default();
        if let Ok(log_level_str) = d.read_struct_field("log_level", 0, |d| d.read_str()) {
            result.log_level = try!(log_level_str
                .parse()
                .map_err(|_| d.error(&format!("unknown log level '{}'", log_level_str))));
        }
        Ok(result)
    }
}

impl Default for LoggingSettings {
    fn default() -> LoggingSettings {
        LoggingSettings {
            log_level: LogLevelFilter::Info,
        }
    }
}

pub struct Settings {
    logging: LoggingSettings,
}

impl Settings {
    fn from_toml(toml: &str) -> Result<Settings> {
        let mut table = try!(toml::Parser::new(toml).parse().ok_or(Error::CannotParse));
        let logging = if let Some(section) = table.remove("logging") {
			try!(toml::decode(section).ok_or(Error::CannotDecode))            
        } else { LoggingSettings::default() };
        Ok(Settings {
			logging: logging                
        })
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            logging: LoggingSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {

	use log::LogLevelFilter;

	use super::*;
	
	#[test]
	fn should_load_defaults_from_empty_toml() {
	    let s = Settings::from_toml("").ok().unwrap();
	    assert_eq!(s.logging.log_level, LogLevelFilter::Info);	    
	}   

	#[test]
	fn should_load_logging_defaults_from_empty_section() {
	    let s = Settings::from_toml(r#"
        	[logging]
        	"#).ok().unwrap();
	    assert_eq!(s.logging.log_level, LogLevelFilter::Info);	    
	}   

	#[test]
	fn should_load_logging_log_level() {
	    let s = Settings::from_toml(r#"
        	[logging]
        	log_level = "DEBUG"
        	"#).ok().unwrap();
	    assert_eq!(s.logging.log_level, LogLevelFilter::Debug);
	    let s = Settings::from_toml(r#"
        	[logging]
        	log_level = "warn"
        	"#).ok().unwrap();
	    assert_eq!(s.logging.log_level, LogLevelFilter::Warn);	    
	    let s = Settings::from_toml(r#"
        	[logging]
        	log_level = "Trace"
        	"#).ok().unwrap();
	    assert_eq!(s.logging.log_level, LogLevelFilter::Trace);
	}   
}