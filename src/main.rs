extern crate rsock;

mod err;
use err::ROLErr;

use std::mem;
use std::{thread, time};

use rsock::rsock::*;
use rsock::consts::*;

use clap::{App, Arg};

/* The process:
 *  Initialize socket var with INVALID_SOCKET ( SOCKET sock = INVALID_SOCKET)
 *  Initialize socket params (iFamily = AF_UNSPEC; iType = ...)
 *  Initialize Winsock structs (WSAData = wsaData {0};
 *                          iResult = WSAStartupe(MAKEWORD(2,2) &wsaData);)
 *  Get a socket : sock = socket(iFamily, iType, iProtocol);
 *  Do some stuff
 *  CloseSocket()
 *  WSACleanup()
 */

fn main()
{
    // Error handling...
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) =>
        {
            eprintln!("{}", err);
            1
        }
    });
}

fn run() -> Result<(), ROLErr>
{
    // Clap argument management
    let args = App::new("RoL - Rust on Lan")
                    .version("0.666")
                    .author("Hrafnskogr <hrafnskogr@pm.me>")
                    .about("Basic program to send magic packets over the LAN")
                    .arg(
                        Arg::with_name("host")
                        .help("MAC address of the remote host to wake up.\n\tFormat:\n\tAABBCCDDEEFF or AA:BB:CC:DD:EE:FF\n\t-- case insensitive --")
                        .index(1)
                        .required(true)
                    )
                    .get_matches();

    send_packet(mac_from_str(args.value_of("host")
                            .unwrap())?
                )?;
    
    Ok(())
}

fn mac_from_str(host: &str) -> Result<Vec<u8>, ROLErr>
{
    // Check what format we have as an input
    // and act accordingly
    let parts: Vec<&str> = match host.contains(":")
    {
        true => host.split(":").collect(),
        false => 
        {
            let mut tmp: Vec<&str> = Vec::new();
            for i in 0..(host.len() / 2)
            {
                tmp.push(&host[i*2..(i*2)+2]);
            }
            tmp
        }
    };

    // Verify that we have a correct MAC address
    match parts.len()
    {
        0..=5 => return Err( ROLErr { code: 2, message: String::from("MAC Addreess too short"), wsa_code: 0 } ),
        6 => (),
        _ => return Err( ROLErr { code: 3, message: String::from("MAC Address too long"), wsa_code: 0 } ),
    }

    // Verify that the MAC address uses only hex chars
    match parts.iter().map(|x| u8::from_str_radix(x, 16)).collect()
    {
        Ok(y) => return Ok(y),
        Err(_) => return Err( ROLErr {code: 1, message: String::from("BADCHAR"), wsa_code: 0} )
    }
}

fn send_packet(mac: Vec<u8>) -> Result<(), ROLErr>
{
    let family: usize = AF_INET;
    let sock_type: usize = SOCK_DGRAM;
    let proto: usize = IPPROTO_UDP;

    // Init WSAData Structure
    let wsa_data: Box<WSAData> = Box::new(WSAData::default());

    // Init WSA
    wsa_startup(make_word(2u16, 2u16), Box::into_raw(wsa_data) as *const usize);

    // Get a socket
    //let mut sock: SOCKET = INVALID_SOCKET;
    let sock = get_socket(family, sock_type, proto);
    if sock == INVALID_SOCKET
    {
        clean(sock);
        return Err(ROLErr {  code: 4,
                             message: format!("Error while running get_socket({}, {}, {})", family, sock_type, proto),
                             wsa_code: wsa_get_last_error(),
                          });
    }

    // Allocate options value on heap
    // Then set socket options
    let opt: Box<bool> = Box::new(true);
    match set_sock_opt(sock, SOL_SOCKET,SO_BROADCAST, Box::into_raw(opt) as *const usize, mem::size_of::<bool>())
    {
        0 => (),
        _ => { return Err(ROLErr { code: 4,
                                message: format!("Error while running set_sock_opt."),
                                wsa_code: wsa_get_last_error(), 
                        }); },
    }

    // Define the destination structure for the send_to function
    let mut dst: Box<sockaddr_in> = Box::new(sockaddr_in::default());
    dst.sin_family = AF_INET as i16;
    dst.sin_port = 9;
    
    // Convert string IP address to in mem structure that winsock will be happy with
    inet_from_str(AF_INET, "255.255.255.255".as_ptr(), dst.sin_addr.as_ptr() as *const usize);

    // Pointerize the destination structure
    let dst_ptr: *const usize = Box::into_raw(dst) as *const usize;
   
    // Build the data needed to be transmitted in the packet
    let data: Vec<u8> = build_packet(mac);

    // Send the packet(s)
    for i in 0..1
    {
        let sent = sock_send_to(sock, &data[0] as *const u8, data.len(), 0, dst_ptr, 16);
        thread::sleep(time::Duration::from_millis(20));
        println!("Byte(s) sent : {} | idx: {}\nData Sent: {:x?}", sent, i, data);
    }

    // Close the socket an gracefully exit the WSA API
    clean(sock);

    Ok(())
}

fn build_packet(dst_mac: Vec<u8>) -> Vec<u8>
{
    // Init empty packet data
    let mut data: Vec<u8> = Vec::new();

    // Add synchronization sequence
    data.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    // Add 16 times the mac address of the remote host
    for _ in 0..16
    {
        data.extend_from_slice(&dst_mac[..]);
    }

    data
}

fn clean(socket: SOCKET)
{
    close_socket(socket);
    wsa_cleanup();
}


#[inline]
fn make_word(h: u16, l: u16) -> u16
{
    (h << 8) + l
}

