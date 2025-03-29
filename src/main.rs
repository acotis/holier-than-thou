
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hole {
    category: String,
    id: String,
    name: String,
    preamble: String,
    links: Vec<HoleLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HoleLink {
    name: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Solution {
    bytes: u32,
    chars: u32,
    golfer: String,
    hole: String,
    lang: String,
    scoring: String,
    submitted: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // The golfers we care about.
    
    let golfers = ["acotis", "lynn", "JayXon"];

    // Get a list of all hole IDs via the API.

    let holes_resp = 
        reqwest::get("http://code.golf/api/holes").await?
        .text().await?;

    let holes: Vec<Hole> = serde_json::from_str(&holes_resp).unwrap();

    // Collect the full solutions log for each hole, in Rust.

    let futures = holes.iter().map(|hole| (async || (hole.id.clone(), get_solution_log(&hole.id).await))());

    println!("Fetching API data...");
    let responses = futures_util::future::join_all(futures).await;
    println!("Done.");

    for response in responses {
        println!("{:35} {:?}", response.0, response.1);
    }

    Ok(())
}

async fn get_solution_log(hole_id: &str) -> Vec<Solution> {
    let url = format!(
        "http://code.golf/api/solutions-log?hole={}&lang=rust",
        urlencoding::encode(hole_id),
    );

    for _attempt in 0..10 {
        let resp = reqwest::get(&url).await.unwrap();
        if !resp.status().is_success() {continue;}
        let text = resp.text().await.unwrap();

        return serde_json::from_str(&text).expect("could not parse solution log");
    }

    panic!("When fetching solutions log for hole \"{hole_id}\", the code.golf API gave a non-2XX status code for 10 attempts in a row. The code.golf API is a little unstable, so you might just try re-running the script.");
}

