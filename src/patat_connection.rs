use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct PatatConnection {
    connection: TcpStream,
}

impl PatatConnection {
    pub fn new(socket: &TcpListener) -> Self {
        let (connection, addr) = socket.accept().unwrap();
        println!("Connection from {}", addr);
        PatatConnection { connection }
    }

    pub fn send_data(&mut self, message_buffer: &[u8]) -> std::io::Result<()> {
        let message_length_buffer = [
            (message_buffer.len() >> 8) as u8,
            (message_buffer.len() & 0xff) as u8,
        ];
        self.connection.write_all(&message_length_buffer)?;
        self.connection.write_all(message_buffer)?;
        Ok(())
    }

    pub fn receive_data(&mut self) -> std::io::Result<Vec<u8>> {
        let mut receive_buffer = [0u8; 2];
        self.connection.read_exact(&mut receive_buffer)?;
        let message_length = ((receive_buffer[0] as u32) << 8) + (receive_buffer[1] as u32);
        println!("Message length {}", message_length);

        let mut message = vec![0u8; message_length as usize];
        self.connection.read(&mut message)?;
        Ok(message)
    }
}
