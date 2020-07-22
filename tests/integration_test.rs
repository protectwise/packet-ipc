use packet_ipc::{AsIpcPacket, Client, Error, IpcPacket, Packet, Server};
use tokio_test::block_on;

#[test]
fn test_roundtrip() {
    let packets = vec![Packet::new(std::time::SystemTime::now(), vec![3u8])];
    let ipc_packets: Vec<_> = packets.iter().map(IpcPacket::from).collect();
    let data = bincode::serialize(&ipc_packets).unwrap();
    let out_packets: Vec<Packet> = bincode::deserialize(data.as_slice()).unwrap();
    assert_eq!(packets[0].data(), out_packets[0].data());
}

#[test]
fn test_connect_to_server() {
    let _ = env_logger::try_init();

    let server = Server::new().expect("Failed to create server");
    let server_name = server.name().clone();

    let client_thread = std::thread::spawn(move || Client::new(server_name));

    let _server_tx = server.accept().expect("Failed to accept connection");

    client_thread
        .join()
        .expect("Failed to join")
        .expect("Failed to connect client");
}

#[test]
fn test_packet_receive() {
    let _ = env_logger::try_init();

    let server = Server::new().expect("Failed to create server");
    let server_name = server.name().clone();

    let client_thread = std::thread::spawn(move || {
        Client::new(server_name).map(|mut cli| {
            let mut packets = vec![];

            packets.push(cli.recv(1));
            packets.push(cli.recv(1));

            packets
        })
    });

    let mut server_tx = server.accept().expect("Failed to accept connection");
    block_on(
    server_tx
        .send(vec![Packet::new(std::time::SystemTime::now(), vec![3u8])]))
        .expect("Failed to send");

    server_tx.close().expect("Failed to close");

    let res = client_thread
        .join()
        .expect("Failed to join")
        .expect("Failed to connect client");
    let res: Result<Vec<_>, Error> = res.into_iter().collect();
    let res = res.expect("Failed to get packets");

    let packets = res[0].as_ref().expect("No message");
    assert_eq!(packets[0].data()[0], 3u8);

    assert!(res[1].is_none());
}

#[test]
fn test_multiple_packet_receive() {
    let _ = env_logger::try_init();

    let server = Server::new().expect("Failed to create server");
    let server_name = server.name().clone();

    let client_thread = std::thread::spawn(move || {
        Client::new(server_name).map(|mut cli| {
            let mut packets = vec![];

            packets.push(cli.recv(1));
            packets.push(cli.recv(1));
            packets.push(cli.recv(1));

            packets
        })
    });

    let mut server_tx = server.accept().expect("Failed to accept connection");

    block_on(server_tx
        .send(vec![Packet::new(std::time::SystemTime::now(), vec![3u8])]))
        .expect("Failed to send");
    block_on(server_tx
        .send(vec![Packet::new(std::time::SystemTime::now(), vec![4u8])]))
        .expect("Failed to send");

    server_tx.close().expect("Failed to close");

    let res = client_thread
        .join()
        .expect("Failed to join")
        .expect("Failed to connect client");
    let res: Result<Vec<_>, Error> = res.into_iter().collect();
    let res = res.expect("Failed to get packets");

    let packets = res[0].as_ref().expect("No message");
    assert_eq!(packets[0].data()[0], 3u8);

    let packets = res[1].as_ref().expect("No message");
    assert_eq!(packets[0].data()[0], 4u8);

    assert!(res[2].is_none());
}
