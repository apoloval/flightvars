//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::boxed::Box;
use std::path::Path;

use log4rs::init_config;
use log4rs::appender::FileAppender;
use log4rs::config::{Appender, Config, Root};

use config::LoggingSettings;

pub fn config_logging(settings: LoggingSettings) {
    init_config(log_config(settings)).unwrap()
}

fn log_config(settings: LoggingSettings) -> Config {
    let log_path = Path::new(&settings.file);
    let file_appender = FileAppender::builder(log_path)
        .pattern(settings.pattern)
        .build()
        .unwrap();
    let main_appender = Appender::builder("main".to_string(), Box::new(file_appender))
        .build();
    let root = Root::builder(settings.level)
        .appender("main".to_string())
        .build();
    let config = Config::builder(root)
        .appender(main_appender)
        .build()
        .unwrap();
    config
}
