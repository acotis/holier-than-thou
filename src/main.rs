
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
    #[serde(default)] rank: usize,
}

struct SolutionLog {
    hole_id: String,
    solutions: Vec<Solution>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // The golfers we care about.
    
    let scoring = "bytes";
    let golfers = ["acotis", "lynn", "JayXon"];
    let timestamp_cutoff = "2026";

    // Get a list of all hole IDs via the API.

    println!();
    println!("Fetching list of holes...");

    let holes_resp = reqwest::get("http://code.golf/api/holes").await?.text().await?;
    let holes: Vec<Hole> = serde_json::from_str(&holes_resp).unwrap();

    // Collect the full solutions log for each hole, in Rust.

    println!("Fetching solution log for each hole (this will take several seconds)...");

    let futures = holes.iter().map(|hole| (async || 
        SolutionLog {
            hole_id: hole.id.clone(), 
            solutions: get_solution_log(&hole.id).await
        }
    )());

    let mut solution_logs = futures_util::future::join_all(futures).await;

    // Active development:

    println!("Processing data...");

    let before = std::time::Instant::now();

    for log in &mut solution_logs {

        // Keep only the solutions with the correct scoring method which
        // were submitted before the cutoff.

        log.solutions.retain(|solution| solution.scoring == scoring);
        log.solutions.retain(|solution| *solution.submitted <= *timestamp_cutoff);

        // Filter down to only each golfer's latest submission.

        log.solutions.sort_by_key(|solution| solution.submitted.clone());
        log.solutions.reverse();
        log.solutions.sort_by_key(|solution| solution.golfer.clone());
        //log.solutions.dedup_by_key(|solution| solution.golfer.clone());

        // Analyze ranks (the API doesn't give us this info directly).

        /*
        match scoring {
            "bytes" => log.solutions.sort_by_key(|solution| solution.bytes),
            "chars" => log.solutions.sort_by_key(|solution| solution.chars),
            _ => panic!(),
        }
        */



        // Other stuff

        //log.solutions.retain(|solution| golfers.contains(&&*solution.golfer));
        //log.solutions.sort_by_key(|solution| solution.submitted.clone());
    }

    let after = std::time::Instant::now();

    println!("Done processing in {}ms.", (after - before).as_millis());
    println!();

    for log in &solution_logs {
        if log.hole_id == "catalan-numbers" {
            for solution in &log.solutions {
                println!(
                    "{:20} {:4}   {}",
                    solution.golfer,
                    solution.bytes,
                    solution.submitted,
                );
            }
        }
    }

    println!();
    println!("{} total solutions", solution_logs.iter().map(|log| log.solutions.len()).sum::<usize>());

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

