
use std::fmt;
use std::error::Error;
use serde::{Serialize, Deserialize};
use regex::Regex;
use clap::Parser;
use chrono::{Utc};

const BOLD:     &'static str = "\x1b[1m";
const DIM:      &'static str = "\x1b[2m";
const ULINE:    &'static str = "\x1b[4m";
const GREEN:    &'static str = "\x1b[32m";
const RED:      &'static str = "\x1b[31m";
const YELLOW:   &'static str = "\x1b[33m";
const BROWN:    &'static str = "\x1b[38;5;130m";
const BLUE:     &'static str = "\x1b[36m";
const GREY:     &'static str = "\x1b[38;5;236m";
const MGREY:    &'static str = "\x1b[38;5;238m";
const LGREY:    &'static str = "\x1b[38;5;240m";
const MLGREY:   &'static str = "\x1b[38;5;242m";
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
    #[serde(alias = "login")] golfer: String,
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
    golfers: Vec<String>,
    scoring: String,
    hole_name_width: usize,
    bar_width: usize,
}

#[derive(Parser)]
struct Arguments {
    me: String,
    them: String,
    #[arg(short, long, default_value="rust" )] lang: String,
    #[arg(short, long, default_value="bytes")] scoring: String,
    #[arg(short, long                       )] cutoff: Option<String>,
    #[arg(       long                       )] reference: Option<String>,
    #[arg(       long, default_value="33"   )] hole_name_width: usize,
    #[arg(       long, default_value="20"   )] score_bar_width: usize,
    #[arg(short, long                       )] reverse: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Parse arguments.

    let args = Arguments::parse();
    let mut golfers = vec![args.me, args.them];

    if let Some(reference) = args.reference {
        golfers.push(reference);
    }

    let cutoff_provided = args.cutoff.is_some();
    let mut cutoff = args.cutoff.unwrap_or(Utc::now().format("%Y-%m-%d").to_string());

    // Validate the date just a little to make it not be a massive
    // UI issue.

    #[derive(PartialEq, Eq)]
    enum CutoffType {IncludeEnd, ExcludeEnd}
    use CutoffType::*;

    let date_regexes = vec![
        (IncludeEnd, Regex::new(r"^\d\d\d\d$").unwrap()),
        (IncludeEnd, Regex::new(r"^\d\d\d\d-\d\d$").unwrap()),
        (IncludeEnd, Regex::new(r"^\d\d\d\d-\d\d-\d\d$").unwrap()),
        (ExcludeEnd, Regex::new(r"^\d\d\d\d-\d\d-\d\d \d\d:\d\d$").unwrap()),
        (ExcludeEnd, Regex::new(r"^\d\d\d\d-\d\d-\d\d \d\d:\d\d:\d\d$").unwrap()),
        (ExcludeEnd, Regex::new(r"^\d\d\d\d-\d\d-\d\d \d\d:\d\d:\d\d.\d+$").unwrap()),
    ];

    let date_format =
        date_regexes
            .iter()
            .find(|(_cutoff_type, regex)| regex.is_match(&cutoff));

    match date_format {
        Some((cutoff_type, _)) => {
            if *cutoff_type == IncludeEnd {
                cutoff += "z";
            }
        },
        None => {
            println!("Invalid date format. Try a date in one of these formats:");
            println!("    — 2025");
            println!("    — 2025-03");
            println!("    — 2025-03-31");
            println!("    — 2025-03-31 12:15");
            println!("    — 2025-03-31 12:15:29");
            println!("    — 2025-03-31 12:15:29.185779");
            return Ok(());
        }
    }

    // Get a list of all hole IDs via the API.

    println!();
    println!("Fetching list of holes...");

    let holes_resp = reqwest::get("http://code.golf/api/holes").await?.text().await?;
    let holes: Vec<Hole> = serde_json::from_str(&holes_resp).unwrap();

    // Collect the full solutions log for each hole in the selected language.

    println!("Fetching solution log for each hole (this will take several seconds)...");

    if cutoff_provided {
        println!("{YELLOW}Warning:{RESET} historical reports generated using the --cutoff flag may include deleted and invalidated solutions");
    }

    let futures = holes.iter().map(|hole| (async || 
        SolutionLog {
            hole_id: hole.id.clone(), 
            solutions: get_solution_log(!cutoff_provided, &args.lang, &hole.id).await,
            gold_length: usize::MAX,
            golfers: golfers.to_vec(),
            scoring: args.scoring.clone(),
            bar_width: 0, // set later
            hole_name_width: 0, // set later
        }
    )());

    let mut solution_logs = futures_util::future::join_all(futures).await;

    // Debug.
    
    /*
    let mut dates: Vec<String> = solution_logs.iter().flat_map(|log| log.solutions.iter().map(|sol| sol.submitted.to_owned())).collect();
    
    dates.push(cutoff.to_owned());
    dates.sort();

    for date in dates {
        if date == cutoff {
            println!("{date} ——————————————————————————————————————————————————————————");
        } else {
            println!("{date}");
        }
    }

    return Ok(());
    */

    // Process the data.

    println!("Processing data...");

    let before = std::time::Instant::now();

    for log in &mut solution_logs {

        // Give each solution an unqualified "length" which is its length
        // in either bytes or chars depending on the scoring method we
        // care about.

        for solution in &mut log.solutions {
            solution.length = match &*args.scoring {
                "bytes" => solution.bytes,
                "chars" => solution.chars,
                _ => panic!("invalid scoring criterion: '{}'", args.scoring),
            }
        }

        // Keep only the solutions with the correct scoring method which
        // were submitted before the cutoff.

        log.solutions.retain(|solution| solution.scoring == args.scoring);
        log.solutions.retain(|solution| solution.submitted <= cutoff);

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

        if log.solutions.len() > 0 {
            log.gold_length = log.solutions[0].length;
        }

        // Keep only the entries from golfers we care about.

        log.solutions.retain(|solution| golfers.contains(&solution.golfer));
    }

    let after = std::time::Instant::now();

    println!("Done processing in {}ms.", (after - before).as_millis());
    println!();
    println!();

    // Keep only the holes for which both <me> and <them> have made submissions.

    solution_logs.retain(|log|
        log.length_for(&golfers[0]) < usize::MAX &&
        log.length_for(&golfers[1]) < usize::MAX
    );

    // Sort by how well <me> is doing compared to <them>, with a backup metric
    // of how well I'm doing on an absolute scale.

    solution_logs.sort_by_key(|log|
        log.sort_score(&golfers[0])
    );

    solution_logs.sort_by_key(|log|
        log.sort_score(&golfers[0]) as isize -
        log.sort_score(&golfers[1]) as isize
    );

    if !args.reverse {
        solution_logs.reverse();
    }

    // Compute a bunch of stuff for formatting.

    let hole_name_width = args.hole_name_width;
    let mut bar_width = args.score_bar_width;

    let wins   = solution_logs.iter().filter(|log| log.length_for(&golfers[0]) <  log.length_for(&golfers[1])).count();
    let draws  = solution_logs.iter().filter(|log| log.length_for(&golfers[0]) == log.length_for(&golfers[1])).count();
    let losses = solution_logs.iter().filter(|log| log.length_for(&golfers[0]) >  log.length_for(&golfers[1])).count();
    let delta  = losses as isize - wins as isize;
    let total  = wins + losses + draws;

    let num_len = |num: usize| if num > 0 {num.ilog(10) + 1} else {1};
    let wdl_width = (num_len(wins) + num_len(draws) + num_len(losses) + 6) as usize;

    // Stupid psychotic hack: fiddle with the width of the scoring bar based
    // on the width of the W/D/L figure, so that it can be perfectly centered
    // no matter what.

    if (wdl_width as isize - bar_width as isize) % 2 != 0 {
        bar_width += 1;
    }

    // Compute more stuff for formatting.

    cutoff = cutoff.replace("z", "");

    let empty  = "";
    let asof   = "as of";
    let indent = hole_name_width - (args.lang.len() + 1 + asof.chars().count() + 1 + cutoff.len());
    let lcenter = (bar_width - wdl_width) / 2;
    let rcenter = ((bar_width - wdl_width) + 1) / 2;

    let names_v1 = format!("{} vs. {}", golfers[0], golfers[1]);
    let names_v2 = format!("{} v. {}", golfers[0], golfers[1]);

    let names = if (names_v1.len() - wdl_width) % 2 == 0 {
        names_v1
    } else {
        names_v2
    };

    let names_indent = (hole_name_width * 2 + 4 + bar_width - names.len()) / 2;

    // Give the SolutionLogs the formatting info they need.

    for log in &mut solution_logs {
        log.hole_name_width = hole_name_width;
        log.bar_width = bar_width;
    }

    // Print the holes.

    for log in &solution_logs {
        println!("{log}");
    }

    // Print the after-summary.

    println!();
    print!("{empty:indent$}{ULINE}{LLGREY}{}{RESET} {LGREY}{asof}{RESET} {LLGREY}{ULINE}{}{RESET}  ", args.lang, cutoff);
    print!("{empty:lcenter$}{GREEN}{wins}{RESET} {LGREY}/{RESET} {LLLGREY}{draws}{RESET} {LGREY}/{RESET} {RED}{losses}{RESET}{empty:rcenter$}  ");

    match delta {
        1..   => print!("{BOLD}{RED}+{delta} loss{}{RESET}", if delta.abs() > 1 {"es"} else {"!"}),
        0     => print!("Tie!!"),
        ..=-1 => print!("{BOLD}{GREEN}+{} win{}!!!{RESET}", -delta, if delta.abs() > 1 {"s!"} else {""}),
    };

    print!(" {MLGREY}({total} holes){RESET}");

    println!();
    println!("{empty:names_indent$}{LLGREY}{names}{RESET}");
    println!();
    println!();

    Ok(())
}

async fn get_solution_log(clean_api: bool, lang: &str, hole_id: &str) -> Vec<Solution> {
    let url = if clean_api {
        format!(
            "http://code.golf/scores/{}/{}/all",
            urlencoding::encode(hole_id),
            urlencoding::encode(lang),
        )
    } else {
        format!(
            "http://code.golf/api/solutions-log?hole={}&lang={}",
            urlencoding::encode(hole_id),
            urlencoding::encode(lang),
        )
    };

    for _attempt in 0..10 {
        let resp = reqwest::get(&url).await.unwrap();
        if !resp.status().is_success() {continue;}
        let text = resp.text().await.unwrap();

        let mut ret: Vec<Solution> = serde_json::from_str(&text).expect("could not parse solution log");

        // Fix up the dates to look like "2025-03-31 12:15:17.129587".

        for sol in &mut ret {
            sol.submitted = sol.submitted.replace("T", " ").replace("Z", "");
        }

        return ret;
    }

    panic!("When fetching solutions log for hole \"{hole_id}\", the code.golf API gave a non-2XX status code for 10 attempts in a row. The code.golf API is a little unstable, so you might just try re-running the script.");
}

impl fmt::Display for SolutionLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{LLLLGREY}{:>1$}{RESET}  ", self.hole_id, self.hole_name_width)?;

        let mut markers: Vec<(String, usize)> = vec![];

        for sol in &self.solutions {
            let sigil = format!(
                "{BOLD}{}{}{RESET}",
                [GREEN, BROWN, BLUE][self.golfers.iter().position(|i|i==&sol.golfer).unwrap()],
                sol.golfer.chars().next().unwrap(),
            );

            let mut shift = (sol.score / 1000.0 * (self.bar_width-1) as f32) as usize;
            while markers.iter().any(|marker| marker.1 == shift) {
                shift -= 1;
            }

            markers.push((sigil, shift));
        }

        for i in 0..self.bar_width {
            write!(
                f, "{}",
                markers.iter()
                       .find(|marker| marker.1 == i)
                       .map(|marker| marker.0.clone())
                       .unwrap_or(format!("{GREY}—{RESET}"))
            )?;
        }

        let delta = self.length_for(&self.golfers[0]) as isize - self.length_for(&self.golfers[1]) as isize;
        match delta {
            ..0 => write!(f, "  {DIM}{GREEN}{delta} {}{}{RESET}", &self.scoring[..4], if delta.abs() > 1 {"s"} else {""})?,
            1.. => write!(f, "  {DIM}{RED}+{delta} {}{}{RESET}",  &self.scoring[..4], if delta.abs() > 1 {"s"} else {""})?,
             0  => write!(f, "  {MLGREY}Tie{RESET}")?,
        };

        write!(
            f, " {MGREY}({}-{}|{}){RESET}",
            self.length_for(&self.golfers[0]),
            self.length_for(&self.golfers[1]),
            self.gold_length,
        )?;

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

