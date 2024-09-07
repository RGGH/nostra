use std::{env, net::{Ipv4Addr, SocketAddr, SocketAddrV4}, time::Duration};
use nostr::{EventBuilder, Filter, Kind };
use nostr_sdk::EventSource;
use std::fs::File;
use std::collections::HashSet;
use crate::bitcoin::XOnlyPublicKey;

use futures_util::stream::StreamExt;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

type Tags = Vec<Vec<String>>;


#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct NosContent {
    content: String,
    pubkey: XOnlyPublicKey,
    created_at: i32,
    sig: String,
    #[serde(rename = "tags")]
    tags: Tags,
    id: EventId
}


fn load_env(){
    dotenv().ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env();
    let private_key = env::var("PRIVATE_KEY").expect("Private key not found");
    let secret_key = SecretKey::from_bech32(private_key)?;
    let my_keys = Keys::new(secret_key);

    // Configure client to use proxy for `.onion` relays
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    let connection: Connection = Connection::new()
        .proxy(addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);

    // Create new client with custom options.
    let client = Client::with_opts(&my_keys, opts);

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;

    // Connect to relays
    client.connect().await;

    
    // Use EventSource::Relays as the event source
    let event_source = EventSource::relays(Some(Duration::from_secs(40)));

        // Build a filter for notes containing the hashtag #jobstr
    let filter = Filter::new()
        .kind(Kind::TextNote)
        .hashtag("jobstr");


     let events = client.get_events_of(vec![filter], event_source).await?;
    // check out to_string_pretty !!
    let json_data = serde_json::to_string_pretty(&events)?;
    println!("{:?}", json_data);
    // Deserialize the JSON data into a vector of structs
    let nos_rawvec: Vec<NosContent> =
        serde_json::from_str(&json_data).expect("Failed to deserialize JSON");

    // Use a HashSet to remove duplicates
    let set: HashSet<_> = nos_rawvec.into_iter().collect();

    // Convert the HashSet back into a vector if needed
    let nos_vec: Vec<_> = set.into_iter().collect();

    for event in events {
         println!("\nevent content = \x1b[42m  \x1b[0m{:#?}", event.content);
    }

    // Create or open the output file
    let mut file = File::create("output.csv")?;

    // Open or create a file for writing
    let mut file = File::create("output.json")?;

    // Serialize the vector of NosCont to JSON and write it to the file
    serde_json::to_writer_pretty(&mut file, &nos_vec)?;

    println!("JSON file created successfully - upsert to jobstr!");



    Ok(())
    // Publish a text note:
    //client.publish_text_note("Experimenting with Rust and nostr_sdk - this was posted with rust-nostr - I'll fix Jobstr.work soon!", []).await?;
    //println!("{:?}", "sent note!\n");
}
