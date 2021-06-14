use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use a2s_parse::{
    info::parse_source_info,
    packet::{is_payload_split, parse_single_packet},
};

extern crate a2s_parse;

fn main() -> () {
    let remote_addr = SocketAddr::from(([208, 103, 169, 70], 27022));

    let info_request = a2s_parse::info::REQUEST_INFO;

    println!("Packet: {:X?}", info_request);

    // Start udp stuff
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket
        .connect(remote_addr)
        .expect("socket connection failed");
    socket.set_read_timeout(Some(Duration::new(5, 0))).unwrap();

    println!("Local: {:?}", socket.local_addr());
    println!("Remote: {:?}", socket.peer_addr());

    let _res = socket.send(&info_request).expect("Failed to send");

    // MTU is usually 1248 but this just gives some room just in case
    let mut buf = vec![0u8; 1600];

    let (size, r_addr) = socket.recv_from(&mut buf).expect("Failed to recv");
    // Trim down to just the recieved data
    buf.resize(size, 0);

    println!("Received: {:X?} from: {:?}", buf, r_addr);

    // Parse the packet type and header:

    if is_payload_split(&buf).expect("Failed") {
        panic!("Should have been a single packet response!");
    }
    // Skip the first 4 bytes as they indicate if the packet is split or not
    let packet = parse_single_packet(&buf[4..]).expect("Failed to parse header and message type");

    println!(
        "INFO: {:?}",
        parse_source_info(packet.payload).expect("Failed to parse info")
    )
}
