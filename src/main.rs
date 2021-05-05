use clap::{Arg, App};
use std::net::{TcpStream, Shutdown};
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
        .version("0.1.4")
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
        let check = &data[55 .. 63];
        let _bracket = &data[63 .. 64];
        
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

/// Coerce byte array into string - without whitespaces
fn read_from_bytes(bytes: &[u8]) -> String {
    return from_utf8(bytes).unwrap().trim().to_string();
}

/// Creates a TcpStream to provided host 
///
/// Loops indefinitely with a timeout to re-connect if connection could not be established (recursion issue may blow up after weeks)
///
/// # Arguments
/// * `host`: hostname or ip with port, e.g. 192.168.90.208:1234
///
/// # TODO
/// Maybe set Option<> Recursion counter and exit after x tries
fn create_stream(host: &String) -> TcpStream {    
    let stream: TcpStream = match TcpStream::connect(host.to_string()) {
        Ok(s) => { log::info!("Connection established."); s },
        Err(e) => { log::error!("Connection failure, retry: {}", e); thread::sleep(time::Duration::from_millis(10000)); create_stream(host) }    
    };
    // timeouts are important because the request blocks and if the scale doesnt send anything we're dead
    let timeout = time::Duration::from_millis(5000);
    stream.set_read_timeout(Some(timeout)).expect("Fail to set read timeout");
    stream.set_write_timeout(Some(timeout)).expect("Fail to set write timeout");
    return stream;
}

/// Stream command to server and read the response
///
/// If a MQTT client is connected, the response is emitted as JSON to the server with hostname as the topic. 
///
/// # Arguments
/// * `host`: hostname or ip with port, e.g. 192.168.90.208:1234
/// * `sleep_time`: Duration how long to sleep between requests
/// * `maybe_cli`: Option struct that may contain mqtt client
fn run_server (host: String, sleep_time: time::Duration, maybe_cli: Option<mqtt::Client>) {             

    // open tcp socket to host
    let mut stream: TcpStream = create_stream(&host);
                
    loop {        
        // be nice don't spam
        thread::sleep(sleep_time);

        // data buffer for the response
        let mut data = [0 as u8; RESPONSE_SIZE];
        
        // command sent to scale, e.g. RM1 read if scale is moving        
        let send = b"<RM1>";  
        //let send = b"<000226.04.2112:32   71     0.0   154.0  -154.0kg T   1   64819>\r\n";
        log::debug!("Send command {} to TcpStream", read_from_bytes(send));
        let bytes_written = match stream.write(send) {
            Ok(b) => b,
            Err(e) => {
                log::error!("Could not write command to stream: {}", e);
                stream = create_stream(&host);                
                continue;
            }            
        };
        log::trace!("Bytes sent: {}", bytes_written);
        // if query is not sent at full-length, raise Error
        if bytes_written < send.len() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, format!("Sent {}/{} bytes", bytes_written, send.len()))).expect("Error");            
        }

        log::trace!("exec: stream.flush()");
        match stream.flush() {
            Ok(_) => (),
            Err(e) => {
                log::error!("Reconnect because could not flush stream: {}", e);
                stream = create_stream(&host);
                continue;
            }
        };
        
        // read data from tcp stream into buffer                
        log::trace!("exec: stream.read(&mut data)");
        let bytes_read = match stream.read(&mut data) {
            Ok(blub) => { log::trace!("exec: ok, read {}", blub); blub },
            // Not ok, maybe connection reset?
            Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                log::info!("Connection Reset. Trying to reconnect stream...");
                stream = create_stream(&host);
                continue;     
            }
            // Not ok but no connection reset? Try to reconnect anyway
            Err(e) => {
                log::error!("There is something wrong: {}", e);
                stream = create_stream(&host);
                continue;
            }
        };

        log::debug!("Read data from stream: {}", read_from_bytes(&data));

        // read vector with the bytes read from stream
        let mut received: Vec<u8> = vec![];
        received.extend_from_slice(&data[..bytes_read]);       
                
        // response is too short, dump to cli, close and reopen stream, skip this loop iteration
        if bytes_read < 64 || &received[0 .. 1] != b"<" || &received[63 .. 64] != b">" {
            log::warn!("Bytes read: {} and response is invalid, dump is: {}", bytes_read, read_from_bytes(&data));
            log::info!("Shutdown stream");
            stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            stream = create_stream(&host);
            continue;
        }       
        
        if bytes_read == 64 {
            // Remove the *** linefeed from stream because it *** the entire ***
            let mut linefeed = [0 as u8; 2];
            let _linefeed_bytes = stream.read(&mut linefeed).unwrap();            
            
            // parse response into struct
            let response = ReadResponse::from_reader(received).unwrap();

            // if mqtt is connected
            if let Some(cli) =  &maybe_cli {
                emit_response(&cli, host.to_string(), &response);
            }
            log::info!("{:?}", &response);            
        }
        
          
    }
}
/// Emit response to mqtt client
///
/// # Arguments
/// * `cli`: MQTT client 
/// * `host`: Hostname, used here as topic
/// * `response`: Read response object
fn emit_response(cli: &mqtt::Client, host: String, response: &ReadResponse) {

    // convert response to json
    let json_response = serde_json::to_string(&response).unwrap();

    // build message to send
    let msg = mqtt::MessageBuilder::new()
        .topic(host)
        .payload(json_response)
        .qos(1)
        .finalize();
    
    // gogo tell them who we are
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
    let sleep: u64 = matches.value_of("interval").unwrap_or("10000").to_string().parse::<u64>().expect("Wrong refresh parameter, try integer value - e.g. \"1000\"");
    let sleep_time = time::Duration::from_millis(sleep);

    // get mqtt parameter
    let mqtt_url = matches.value_of("mqtt").unwrap_or("false").to_string();
    
    env::set_var("RUST_LOG", matches.value_of("log").unwrap_or("info"));
    env_logger::init();

    // run forever
    if mqtt_url == "false" {
        log::info!("Establish connection to {} without MQTT emission. Refresh rate: {}", host, sleep);
        run_server(host.to_string(), sleep_time, None);        
    } else {
        log::info!("Establish connection to {} with MQTT broker at {}. Refresh rate: {}", host, mqtt_url, sleep);
        let cli = mqtt::Client::new(mqtt_url).unwrap_or_else(|err| { log::error!("Error creating the MQTT client: {:?}", err); process::exit(1)});
    
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
}
