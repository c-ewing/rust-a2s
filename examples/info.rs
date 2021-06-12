use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

extern crate a2s_parse;

fn main() -> () {
    let remote_addr = SocketAddr::from(([208, 103, 169, 70], 27022));

    let info_request = b"TSource Engine Query\0";
    // need the packet header as well
    let mut header = [0xFF, 0xFF, 0xFF, 0xFF].to_vec();
    header.extend(info_request);

    println!("Packet: {:X?}", header);

    // Start udp stuff
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket
        .connect(remote_addr)
        .expect("socket connection failed");
    socket.set_read_timeout(Some(Duration::new(5, 0))).unwrap();

    println!("Local: {:?}", socket.local_addr());
    println!("Remote: {:?}", socket.peer_addr());

    let _res = socket.send(&header).expect("Failed to send");

    // MTU is usually 1248 but this just gives some room just in case
    let mut buf = vec![0u8; 1600];

    let (size, r_addr) = socket.recv_from(&mut buf).expect("Failed to recv");
    // Trim down to just the recieved data
    buf.resize(size, 0);

    println!(
        "Received: {} from: {:?}",
        String::from_utf8_lossy(&buf),
        r_addr
    );
    println!("Raw Buffer: {:?}", buf);
    // Skip the packet type + header for now
    // TODO: Show the full parsing steps
    let info = a2s_parse::info::parse_source_info(&buf[5..]).expect("Failed to parse info");

    println!("INFO: {:?}", info);
}