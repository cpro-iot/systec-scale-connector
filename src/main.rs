use clap::{Arg, App};
use std::net::{TcpStream};
use std::io::{Read, Write, self};
use std::str::from_utf8;
use std::{thread, time, env, process};
use std::vec::Vec;
use env_logger;
use log;
use serde::{Deserialize, Serialize};

extern crate paho_mqtt as mqtt;

const RESPONSE_SIZE: usize = 64;

// use clap to parse CLI arguments
fn check_cli<'a>() -> clap::ArgMatches<'a> {
    let matches = App::new("Cpro IoT Connector for SysTec Scales")
        .version("0.0.1")
        .author("Cpro IoT Connect GmbH - <christian.spaniol@cpro-iot.com>")
        .about("Connect to scales to retrieve live data")
        .arg(Arg::with_name("ip")
                    .short("h")
                    .long("host")
                    .takes_value(true)
                    .help("Hostname or IP Address")
                    .required(true)
                    .index(1))
        .arg(Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .takes_value(true)
                    .help("Service port, defaults to \"1234\""))
        .arg(Arg::with_name("interval")
                    .short("i")
                    .long("interval")
                    .takes_value(true)
                    .help("Refresh interval in milliseconds, defaults to 1000 (1 second)"))
        .arg(Arg::with_name("log")
                    .short("l")
                    .long("log")
                    .takes_value(true)
                    .help("Set Log level"))
        .arg(Arg::with_name("mqtt")
                    .short("m")
                    .long("mqtt")
                    .takes_value(true)
                    .help("MQTT Server and port, e.g. 192.168.93.97:1500"))
        .get_matches();
    
    return matches;
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadResponse {
    error_code: String,
    scale_in_move: String,
    gross_negative: String,
    date: String,
    time: String,
    ident: String,
    scale_nr: String,
    gross: String,
    tara: String,
    net: String,
    unit: String,
    tara_code: String,
    scale_area: String,
    terminal: String,
    check: String
}

impl ReadResponse {
    // beautiful spaghetti to read scale response into struct
    fn from_reader(data: Vec<u8>) -> io::Result<Self> {                
        let _bracket = &data[0 .. 1];
        let error_code = &data[1 .. 3];
        let scale_in_move = &data[3 .. 4];
        let gross_negative = &data[4 .. 5];
        let date = &data[5 .. 13];
        let time = &data[13 .. 18];
        let ident = &data[18 .. 22];
        let scale_nr = &data[22 .. 23];
        let gross = &data[23 .. 31];
        let tara = &data[31 .. 39];
        let net = &data[39 .. 47];
        let unit = &data[47 .. 49];
        let tara_code = &data[49 .. 51];
        let scale_area = &data[51 .. 52];
        let terminal = &data[52 .. 55];
        let check = &data[55 .. 62];
        let _bracket = &data[62 .. 63];
        
        let error_code = read_from_bytes(&error_code);
        let scale_in_move = read_from_bytes(&scale_in_move);
        let gross_negative = read_from_bytes(&gross_negative);
        let date = read_from_bytes(&date);
        let time = read_from_bytes(&time);
        let ident = read_from_bytes(&ident);
        let scale_nr = read_from_bytes(&scale_nr);
        let gross = read_from_bytes(&gross);
        let tara = read_from_bytes(&tara);
        let net = read_from_bytes(&net);
        let unit = read_from_bytes(&unit);
        let tara_code = read_from_bytes(&tara_code);
        let scale_area = read_from_bytes(&scale_area);
        let terminal = read_from_bytes(&terminal);
        let check = read_from_bytes(&check);        
 
        Ok(ReadResponse { 
                error_code, scale_in_move, gross_negative, date, time, ident, scale_nr, gross, tara,
                net, unit, tara_code, scale_area, terminal, check
        })
    }
}

// coerce byte array into string - without whitespaces
fn read_from_bytes(bytes: &[u8]) -> String {
    return from_utf8(bytes).unwrap().trim().to_string();
}


fn run_server (host: String, sleep_time: time::Duration, maybe_cli: Option<mqtt::Client>) {             

    // open tcp socket to host
    let mut stream: TcpStream = TcpStream::connect(host.to_string()).expect("Could not connect");    
                
    loop {
        thread::sleep(sleep_time);
        // data buffer for the response
        let mut data = [0 as u8; RESPONSE_SIZE];
        
        // command sent to scale, e.g. RM1 read if scale is moving        
        let send = b"<RM1>";  
        //let send = b"<000226.04.2112:32   71     0.0   154.0  -154.0kg T   1   64819>\r\n";
        log::debug!("Send command {} to TcpStream", read_from_bytes(send));
        let bytes_written = stream.write(send).unwrap();

        // if query is not sent at full-length, raise Error
        if bytes_written < send.len() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, format!("Sent {}/{} bytes", bytes_written, send.len()))).expect("Error");            
        }
        stream.flush().unwrap();
        
        // read data from tcp stream into buffer        
        let bytes_read = stream.read(&mut data).unwrap();
        log::debug!("Read data from stream: {}", read_from_bytes(&data));

        // read vector with the bytes read from stream
        let mut received: Vec<u8> = vec![];
        received.extend_from_slice(&data[..bytes_read]);       
                
        // response is too short, dump to cli and skip this loop iteration
        if bytes_read < 64 {
            log::warn!("Bytes read: {}, so response is non-valid, dump is: {}", bytes_read, read_from_bytes(&data));            
            continue;
        }       
        
        if bytes_read == 64 {
            // Remove the *** linefeed from stream because it *** the entire ***
            let mut linefeed = [0 as u8; 2];
            let _linefeed_bytes = stream.read(&mut linefeed).unwrap();
            let _linefeed_bytes = stream.read(&mut linefeed).unwrap();
            
            // parse response into struct
            let response = ReadResponse::from_reader(received).unwrap();

            if let Some(cli) =  &maybe_cli {
                emit_response(&cli, host.to_string(), &response);
            }
            log::info!("{:?}", &response);            
        }
        
          
    }
}

fn emit_response(cli: &mqtt::Client, host: String, response: &ReadResponse) {

    let json_response = serde_json::to_string(&response).unwrap();

    let msg = mqtt::MessageBuilder::new()
        .topic(host)
        .payload(json_response)
        .qos(1)
        .finalize();
    
    if let Err(e) = cli.publish(msg) {
       log::error!("Error sending message: {:?}", e);
    }
}

fn main() {

    // Set command line parameters
    let matches = check_cli();

    
    let ip = matches.value_of("ip").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("1234");    

    // create hostname 
    let host = format!("{}:{}", ip, port);

    // refresh interval in milliseconds
    let sleep: u64 = matches.value_of("interval").unwrap_or("1000").to_string().parse::<u64>().expect("Wrong refresh parameter, try integer value - e.g. \"1000\"");
    let sleep_time = time::Duration::from_millis(sleep);

    // get mqtt parameter
    let mqtt_url = matches.value_of("mqtt").unwrap_or("false").to_string();//.expect("Error with MQTT parameter");

    
    env::set_var("RUST_LOG", matches.value_of("log").unwrap_or("info"));
    env_logger::init();

    // run forever
    if mqtt_url == "false" {
        log::info!("Establish connection without MQTT emission");
        run_server(host.to_string(), sleep_time, None);        
    } else {
        log::info!("Connect MQTT Emitter to broker at {}", mqtt_url);
        let cli = mqtt::Client::new(mqtt_url).unwrap_or_else(|err| { log::error!("Error creating the client: {:?}", err); process::exit(1)});
    
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(time::Duration::from_secs(30))
            .connect_timeout(time::Duration::from_secs(25))
            .clean_session(true)
            .finalize();
    
        if let Err(e) = cli.connect(conn_opts) {
            log::error!("Unable to connect: {:?}", e);
            process::exit(1);
        }
    
        run_server(host.to_string(), sleep_time, Some(cli));        
    }


    log::info!("Open Stream to Host: {} Refresh rate: {} ms", host, sleep);

    
}
