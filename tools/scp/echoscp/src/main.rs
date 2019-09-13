extern crate clap;
use clap::{App, Arg};
use dicom_ul::asso::{Association, AssociationOptions};
use dicom_ul::error::Result;
use dicom_ul::{NetStream, DEFAULT_PORT_SECONDARY};
use std::net::{TcpListener, TcpStream};
use dicom_ul::dimse::Command;

fn run(scu_stream: TcpStream) -> Result<()> {
    let mut scu_stream = NetStream::Tcp(scu_stream);
    let mut options = AssociationOptions::default();
    options.add_supported_context(
        "1.2.840.10008.1.1".to_string(),
        vec![
            "1.2.840.10008.1.2".to_string(),
            "1.2.840.10008.1.2.1".to_string(),
        ],
        false,
    )?;
    let mut association = Association::receive_association(&mut scu_stream, options)?;

    let mut process_command = || -> Result<()> {
        let command = association.read_dimse_command()?;

        match command.command {
            Command::EchoRq(data) => {
                //association.send_dimse_response(Command::EchoRq(data));
            }
        }

        Ok(())
    };


    loop {
        if let Err(e) = process_command() {
            association.abort_association()?;
            return Err(e);
        }
    }

    Ok(())
}

fn main() {
    let default_port_number = DEFAULT_PORT_SECONDARY.to_string();
    let matches = App::new("echoscp")
        .arg(
            Arg::with_name("listen-port")
                .help("The destination host name (SCP)")
                .default_value(&default_port_number)
                .required(true)
                .index(1),
        )
        .get_matches();

    let listen_addr = format!("0.0.0.0:{}", matches.value_of("listen-port").unwrap());

    let listener = TcpListener::bind(&listen_addr).unwrap();
    println!("listening on: {}", listen_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match run(stream) {
                Err(e) => {
                    println!("error: {}", e);
                }
                _ => {}
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
