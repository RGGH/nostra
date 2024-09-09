use crate::bitcoin::XOnlyPublicKey;
use dotenv::dotenv;
use nostr::{Filter, Kind};
use nostr_sdk::prelude::*;
use nostr_sdk::EventSource;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

type Tags = Vec<Vec<String>>;

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct NosContent {
    content: String,
    pubkey: XOnlyPublicKey,
    created_at: i32,
    sig: String,
    #[serde(rename = "tags")]
    tags: Tags,
    id: EventId,
}

fn load_env() {
    dotenv().ok();
}

// Calculate the current Unix timestamp in seconds
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

async fn check_for_new_events(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let five_minutes_ago = current_unix_timestamp() - 5 * 60;
    
    // Build a filter for notes containing the hashtag #jobstr
    let filter = Filter::new()
        .kind(Kind::TextNote)
        .hashtag("jobstr")
        .since(five_minutes_ago.into());

    let event_source = EventSource::relays(Some(Duration::from_secs(40)));
    let events = client.get_events_of(vec![filter], event_source).await?;
    
    // Convert events to JSON and print
    let json_data = serde_json::to_string_pretty(&events)?;
    println!("{:?}", json_data);

    // Deserialize the JSON data into a vector of structs
    let nos_rawvec: Vec<NosContent> =
        serde_json::from_str(&json_data).expect("Failed to deserialize JSON");

    // Use a HashSet to remove duplicates
    let set: HashSet<_> = nos_rawvec.into_iter().collect();
    let nos_vec: Vec<_> = set.into_iter().collect();

    for event in events {
        println!("\nevent content = \x1b[42m  \x1b[0m{:#?}", event.content);
    }

    // Create or open the output file
    let mut file = File::create("output.json")?;
    serde_json::to_writer_pretty(&mut file, &nos_vec)?;

    println!("JSON file created successfully - next: upsert to jobstr!");

    Ok(())
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
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    loop {
        check_for_new_events(&client).await?;

        // Sleep for 5 minutes (300 seconds)
        sleep(Duration::from_secs(300)).await;
    }
}
