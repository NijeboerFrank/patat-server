mod evidence;
mod patat_connection;
mod patat_participant;
mod server;

use anyhow::Result;
use evidence::{get_evidence, EvidenceProof};

fn main() -> Result<()> {
    println!("Starting the UDP server...");
    let mut patat_server = server::Server::new();

    loop {
        let _ = &patat_server
            .run_server()
            .expect("Something went wrong on the server");

        let (lemma, path) = get_evidence();

        let e = EvidenceProof::new(path, lemma);
        let e_bytes: Vec<u8> = e.into();

        let another_e: EvidenceProof = e_bytes.into();
	let valid = another_e.valid();
        println!("{}", valid);
	if !valid {
	    break;
	}
    }

    Ok(())
}
