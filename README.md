# packet-ipc

[![build status][travis-badge]][travis-url]
[![crates.io version][crates-badge]][crates-url]
[![docs.rs docs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

Library to share packets between processes using [servo's ipc-channel](https://github.com/servo/ipc-channel).

Attempts to be as efficient as possible while still allowing packets to be used with C FFI.

A packet is defined for this library as any structure which implements `AsIpcPacket`.

[travis-badge]: https://img.shields.io/travis/dbcfd/packet-ipc/master.svg?style=flat-square
[travis-url]: https://travis-ci.com/dbcfd/packet-ipc.svg?branch=master
[crates-badge]: https://img.shields.io/crates/v/packet-ipc.svg?style=flat-square
[crates-url]: https://crates.io/crates/packet-ipc
[docs-badge]: https://img.shields.io/badge/docs.rs-latest-blue.svg?style=flat-square
[docs-url]: https://docs.rs/packet-ipc
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square
[mit-url]: LICENSE-MIT

## Usage

A server must be created before a client, since the client will use the server's name to connect. 

To start, create a server, and accept a connection:

```rust
let server = Server::new().expect("Failed to build server");
let server_name = server.name.clone();
let connection = futures::spawn(server.accept()).expect("No connection formed");
```

Once a server is created, you can then use the server's name to create a client:

```rust
let client = Client::new(server_name.clone()).expect("Failed to connect");
```

At this point, you can send packets to the client:

```rust
connection.send(Some(packets)).expect("Failed to send");
```

or tell the client you are done sending packets:

```rust
connection.send(None).expect("Failed to send");
```

and close the connection:

```rust
connection.close();
```

The client is immediately available for use, and can receive packets using:

```rust
let opt_packets = client.receive_packets(size).expect("Failed to receive packets");
if Some(packets) = opt_packets {
    process_packets(packets);
} //else server is closed
```

## Streaming Packets to Client
Once a connection is formed, it can be used with a stream of packets:

```rust
let stream_result = packets_stream.transfer_ipc(connection).collect().wait();
```
