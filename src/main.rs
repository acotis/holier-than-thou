
use std::fmt;
use std::error::Error;
use serde::{Serialize, Deserialize};

const BOLD:     &'static str = "\x1b[1m";
const DIM:      &'static str = "\x1b[2m";
const ULINE:    &'static str = "\x1b[4m";
const GREEN:    &'static str = "\x1b[32m";
const RED:      &'static str = "\x1b[31m";
const BROWN:    &'static str = "\x1b[38;5;130m";
const BLUE:     &'static str = "\x1b[36m";
const GREY:     &'static str = "\x1b[38;5;236m";
const LGREY:    &'static str = "\x1b[38;5;240m";
const LLGREY:   &'static str = "\x1b[38;5;244m";
const LLLGREY:  &'static str = "\x1b[38;5;252m";
const LLLLGREY: &'static str = "\x1b[38;5;254m";
const RESET:    &'static str = "\x1b[0m";

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
    bytes: usize,
    chars: usize,
    golfer: String,
    hole: String,
    lang: String,
    scoring: String,
    submitted: String,

    #[serde(default)] length: usize,    // Copy of bytes or chars.
    #[serde(default)] rank: usize,      // Computed by us.
    #[serde(default)] score: f32,       // Computed by us.
}

struct SolutionLog {
    hole_id: String,
    solutions: Vec<Solution>,
    gold_length: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // The golfers we care about.
    
    let scoring = "bytes";
    let golfers = ["acotis", "lynn", "JayXon"];
    let timestamp_cutoff = "current moment";
    //let timestamp_cutoff = "2024-10-11T18:50";
    //let timestamp_cutoff = "2024-10-12";
    //let timestamp_cutoff = "2025-04";

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
            solutions: get_solution_log(&hole.id).await,
            gold_length: usize::MAX,
        }
    )());

    let mut solution_logs = futures_util::future::join_all(futures).await;

    // Process the data.

    println!("Processing data...");

    let before = std::time::Instant::now();

    for log in &mut solution_logs {

        // Give each solution an unqualified "length" which is its length
        // in either bytes or chars depending on the scoring method we
        // care about.

        for solution in &mut log.solutions {
            solution.length = match scoring {
                "bytes" => solution.bytes,
                "chars" => solution.chars,
                _ => panic!("invalid scoring criterion: '{scoring}'"),
            }
        }

        // Keep only the solutions with the correct scoring method which
        // were submitted before the cutoff.

        log.solutions.retain(|solution| solution.scoring == scoring);
        log.solutions.retain(|solution| *solution.submitted <= *timestamp_cutoff);

        // Filter down to only each golfer's best submission. This gives
        // us the submissions which were "active" at the cutoff time.

        log.solutions. sort_by_key(|solution| solution.length);
        log.solutions. sort_by_key(|solution| solution.golfer.clone());
        log.solutions.dedup_by_key(|solution| solution.golfer.clone());

        // Sort the solutions and assign ranks, scores, and medals to them.
        // This recreates the leaderboard as-it-was in its entirety.

        log.solutions.sort_by_key(|solution| solution.submitted.clone());
        log.solutions.sort_by_key(|solution| solution.length);

        for i in 0..log.solutions.len() {
            log.solutions[i].score =
                log.solutions[0].length as f32 /
                log.solutions[i].length as f32 *
                1000.0;

            log.solutions[i].rank = 
                if i > 0 && log.solutions[i].length == log.solutions[i-1].length {
                    log.solutions[i-1].rank
                } else {
                    i + 1
                };
        }

        if log.solutions.len() > 1 
        && log.solutions[0].length < log.solutions[1].length {
            log.solutions[0].rank = 0;
        }

        log.gold_length = log.solutions[0].length;

        // Keep only the entries from golfers we care about.

        log.solutions.retain(|solution| golfers.contains(&&*solution.golfer));
    }

    let after = std::time::Instant::now();

    println!("Done processing in {}ms.", (after - before).as_millis());
    println!();

    // Pretty printing.

    solution_logs.retain(|log|
        log.length_for("acotis") < usize::MAX &&
        log.length_for("lynn") < usize::MAX
    );

    solution_logs.sort_by_key(|log|
        log.sort_score("acotis") as isize -
        log.sort_score("lynn") as isize
    );

    solution_logs.reverse();

    for log in &solution_logs {
        println!("{log}");
    }

    // Summary.

    let wins   = solution_logs.iter().filter(|log| log.length_for("acotis") <  log.length_for("lynn")).count();
    let draws  = solution_logs.iter().filter(|log| log.length_for("acotis") == log.length_for("lynn")).count();
    let losses = solution_logs.iter().filter(|log| log.length_for("acotis") >  log.length_for("lynn")).count();
    let delta  = losses as isize - wins as isize;

    let empty  = "";
    let ssb    = "Summary as of";
    let indent = 33 - (ssb.len() + 1 + timestamp_cutoff.len());
    let wdl_width = (wins.ilog(10) + draws.ilog(10) + losses.ilog(10) + 3 + 6) as usize;
    let lcenter = (21 - wdl_width) / 2;
    let rcenter = ((21 - wdl_width) + 1) / 2;

    println!();
    print!("{empty:indent$}{LGREY}{ssb}{RESET} {LLGREY}{ULINE}{timestamp_cutoff}{RESET}  ");
    print!("{empty:lcenter$}{GREEN}{wins}{RESET} {LGREY}/{RESET} {LLLGREY}{draws}{RESET} {LGREY}/{RESET} {RED}{losses}{RESET}{empty:rcenter$}  ");

    match delta {
        1..   => print!("{BOLD}{RED}+{delta} loss{}{RESET}", if delta.abs() > 1 {"es"} else {""}),
        0     => print!("Tie!!"),
        ..=-1 => print!("{BOLD}{GREEN}+{} win{}!!!{RESET}", -delta, if delta.abs() > 1 {"s"} else {""}),
    };

    println!();
    println!();

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

impl fmt::Display for SolutionLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{LLLLGREY}{:>33}{RESET}  ", self.hole_id)?;

        let line_width = 20;
        let mut markers: Vec<(String, usize)> = vec![];

        for sol in &self.solutions {
            let sigil = match sol.golfer.as_str() {
                "acotis" => format!("{BOLD}{GREEN}a{RESET}"),
                "lynn"   => format!("{BOLD}{BROWN}l{RESET}"),
                "JayXon" => format!("{BOLD}{BLUE }J{RESET}"),
                _        => format!("g"),
            };

            let mut shift = (sol.score / 1000.0 * line_width as f32) as usize;
            while markers.iter().any(|marker| marker.1 == shift) {
                shift -= 1;
            }

            markers.push((sigil, shift));
        }

        for i in 0..line_width+1 {
            write!(
                f, "{}",
                markers.iter()
                       .find(|marker| marker.1 == i)
                       .map(|marker| marker.0.clone())
                       .unwrap_or(format!("{GREY}—{RESET}"))
            )?;
        }

        let delta = self.length_for("acotis") as isize - self.length_for("lynn") as isize;
        match delta {
            ..0 => write!(f, "  {DIM}{GREEN}{delta} byte{}{RESET} {GREY}({}){RESET}", if delta.abs() > 1 {"s"} else {""}, self.gold_length)?,
             0  => write!(f, "  {LGREY}Tie{RESET} {GREY}({}){RESET}", self.gold_length)?,
            1.. => write!(f, "  {DIM}{RED}+{delta} byte{}{RESET} {GREY}({}){RESET}", if delta.abs() > 1 {"s"} else {""}, self.gold_length)?,
        };

        Ok(())
    }
}

impl SolutionLog {
    fn sort_score(&self, golfer: &str) -> usize {
        self.solutions
            .iter()
            .find(|solution| solution.golfer == golfer)
            .map(|solution|
                (solution.score * 10000.0) as usize
                + if solution.rank == 0 {1} else {0}
            )
            .unwrap_or(0)
    }

    fn length_for(&self, golfer: &str) -> usize {
        self.solutions
            .iter()
            .find(|solution| solution.golfer == golfer)
            .map(|solution| solution.length)
            .unwrap_or(usize::MAX)
    }
}

