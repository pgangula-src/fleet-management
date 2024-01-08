use clap::{Arg, ArgMatches, Command};
use fms_proto::fms::VehicleStatus;
use log::{debug, error, info, warn};
use zenoh::config::Config;
use zenoh::prelude::sync::*;
use protobuf::Message;

use std::sync::Arc;
use std::time::{Duration, SystemTime};
use zenoh::publication::Publisher;
use zenoh::subscriber::Subscriber;
use std::thread::sleep;
use async_trait::async_trait;
use crate::{status_publishing::StatusPublisher};
use crate::config::WhatAmI;
use crate::config::EndPoint;


const key_expr: &str = "fms/vehicleStatus";


pub fn add_command_line_args(command: Command) -> Command {
    command
        .arg(
            Arg::new("mode")
                 .value_parser(clap::value_parser!(WhatAmI))
                .long("mode")
                .short('m')
                .help("-m, --mode=[MODE] 'The zenoh session mode (peer by default).")
                //.value_parser(["peer", "client"])
                .required(false),
        )
        .arg(
            Arg::new("connect")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .long("connect")
                .short('e')
                .help("-e, --connect=[ENDPOINT]...  'Endpoints to connect to.'")
                .required(false),
        )
        .arg(
            Arg::new("listen")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .long("listen")
                .short('l')
                .help("-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'")
                .required(false),
        )
       .arg(
            Arg::new("no-multicast-scouting")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .long("no-multicast-scouting")
                .help("--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'")
                .required(false),
        )
 }
 
 pub fn parse_args(args: &ArgMatches) -> Config {
     let mut config: Config = Config::default();
     
     if let Some(mode) = args.get_one::<WhatAmI>("mode"){
        config.set_mode(Some(*mode)).unwrap();
    }
  
    if let Some(values) = args.get_many::<String>("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.get_many::<String>("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.get_one::<bool>("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(*values)).unwrap();
    }
    
       
    config
 }

pub struct ZenohPublisher<'a> {
    // Session
    z_session: Arc<Session>,

    // publisher
    publisher: Publisher<'a>,
}

 
 impl<'a> ZenohPublisher<'a> {
    pub async fn new(z_session: Arc<Session>) -> Result<ZenohPublisher<'a>, Box<dyn std::error::Error>>{  

        let publisher = z_session.declare_publisher(key_expr).res().unwrap();
        Ok(ZenohPublisher {

            // Session
            z_session,

            // publisher
            publisher,
        })
    }
 }

    

#[async_trait]
impl<'a> StatusPublisher for ZenohPublisher<'a>{
    async fn publish_vehicle_status(&self, vehicle_status: &VehicleStatus) {
        match vehicle_status.write_to_bytes() {
            Ok(payload) => {
                match self.publisher.put(payload).res() {
                    Ok(_t) => debug!(
                        "successfully published vehicle status to Zenoh",
                    ),
                    Err(e) => {
                        warn!(
                            "error publishing vehicle status to Zenoh: {}",
                             e
                        );
                    }
                };
                return;
            }
            Err(e) => warn!(
                "error serializing vehicle status to protobuf message: {}",
                e
            ),
        }
        
	}
}



