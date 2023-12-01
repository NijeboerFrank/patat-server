use anyhow::Result;
use snow::{Builder, Keypair, TransportState};

use crate::{patat_connection::PatatConnection, patat_participant::PatatParticipant};

pub struct Server {
    protocol_builder: Option<Builder<'static>>,
    server_keypair: Keypair,
}

impl Server {
    pub fn new() -> Self {
        let (protocol_builder, server_keypair) = Self::setup().unwrap();
        Server {
            protocol_builder: Some(protocol_builder),
            server_keypair,
        }
    }

    pub fn run_server(mut self) -> Result<()> {
        self.write_keys_to_file()?;
        let mut connection = PatatConnection::new(65432);

        // Now we can go to the Transport mode since the handshake is done
        let mut transport = self.run_handshake(&mut connection);
        let message = self
            .receive_message(&mut transport, &mut connection)
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(&message));
        self.transfer_message(b"hello", &mut transport, &mut connection)
            .unwrap();

        let merkle_proof = self
            .receive_message(&mut transport, &mut connection)
            .unwrap();
        println!("merkle root {:?}", merkle_proof);

        Ok(())
    }

    fn run_handshake(&mut self, connection: &mut PatatConnection) -> TransportState {
        // Setup the handshake protocol
        let mut protocol = self
            .protocol_builder
            .take()
            .unwrap()
            // Hardcode private key for testing
            .local_private_key("very-secure-password-for-frieten".as_bytes())
            .build_responder()
            .expect("Could not start protocol");

        // -> e, es
        let message = &connection.receive_data().expect("Could not receive data");
        let mut payload_buffer = vec![0u8; 65535];
        let _payload_length = protocol
            .read_message(message, &mut payload_buffer)
            .expect("Couldn't process message");

        // <- e, ee
        let mut buf = vec![0u8; 65535];
        let message_len = protocol
            .write_message(&[1], &mut buf)
            .expect("Something went wrong with creating a new message");
        connection.send_data(&buf[..message_len]).unwrap();

        // -> s, se
        let message = &connection.receive_data().expect("Could not receive data");
        let mut payload_buffer = vec![0u8; 65535];
        let _payload_length = protocol
            .read_message(message, &mut payload_buffer)
            .expect("Couldn't process message");

        println!("Setup handshake");

        // Move to transport mode
        protocol.into_transport_mode().unwrap()
    }
}

impl PatatParticipant for Server {
    fn key_filenames() -> (&'static str, &'static str) {
        ("server.key", "server.key.pub")
    }

    fn keypair(&self) -> &Keypair {
        &self.server_keypair
    }
}
