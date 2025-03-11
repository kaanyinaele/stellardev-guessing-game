use std::{io, process, fs};
use std::time::Instant;
use rand::Rng;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

const HIGHSCORE_FILE: &str = "highscores.json";

#[derive(Serialize, Deserialize, Debug)]
struct HighScore {
    attempts: u32,
    seconds: f64,
    difficulty: u32,
    date: DateTime<Local>,
}

/// Main game loop and logic
/// - Generates secret number based on difficulty
/// - Handles user input and game flow
/// - Returns io::Result for proper error propagation
fn main() -> io::Result<()> {
    println!("Number Guessing Game!");
    
    loop {  // Outer game loop
        // Load and display high scores
        display_high_scores()?;
        
        // Get difficulty and generate secret number
        let difficulty = choose_difficulty();
        let secret = generate_secret_number(difficulty);
        
        let start_time = Instant::now();
        let mut attempts = 0;
        
        // Inner loop for current round
        loop {
            println!("Enter your guess (1-{}) or 'q' to quit:", difficulty);
            
            // Handle input with error recovery
            let guess = match get_guess() {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    continue;
                }
            };

            attempts += 1;

            // Compare guess to secret number
            match guess.cmp(&secret) {
                Ordering::Less => println!("The Secret Number is Higher!"),
                Ordering::Greater => println!("The Secret Number is Lower!"),
                Ordering::Equal => {
                    let duration = start_time.elapsed();
                    println!("Correct! The secret number was {}", secret);
                    println!("You took {} attempts and {:.2} seconds!", 
                        attempts, duration.as_secs_f64());
                    
                    // Save high score
                    save_score(HighScore {
                        attempts,
                        seconds: duration.as_secs_f64(),
                        difficulty,
                        date: Local::now(),
                    })?;
                    
                    display_high_scores()?;
                    break;  // Break inner loop to ask about playing again
                }
            }
        }

        // Ask if player wants to continue
        if !ask_play_again()? {
            println!("Thanks for playing!");
            break;  // Break outer loop to end game
        }
    }
    Ok(())
}

/// Handles user input and validation
/// - Reads line from stdin
/// - Processes quit commands (q/quit)
/// - Validates number format and range
/// - Returns io::Result<u32> for error handling
fn get_guess() -> io::Result<u32> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Normalize input and check for quit commands
    let input = input.trim().to_lowercase();
    if input == "q" || input == "quit" {
        println!("Goodbye!");
        process::exit(0);
    }

    // Chain validation steps: parse then range check
    input.parse()
        .map_err(|e| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Please enter a valid number between 1-100 ({})", e)
        ))
        .and_then(|n| {
            if (1..=100).contains(&n) {
                Ok(n)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Number must be between 1-100"
                ))
            }
        })
}

/// Prompts user to choose difficulty level
/// Returns the maximum range for the chosen difficulty
fn choose_difficulty() -> u32 {
    loop {
        println!("Choose a difficulty level:");
        println!("1. Easy (1 - 50)");
        println!("2. Medium (1 - 100)");
        println!("3. Hard (1 - 200)");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(1) => return 50,
            Ok(2) => return 100,
            Ok(3) => return 200,
            _ => println!("Invalid input. Please enter a valid difficulty level (1, 2, 3).")
        }
    }
}

/// Generates a random number within the specified range
/// Returns the generated number
fn generate_secret_number(range: u32) -> u32 {
    rand::thread_rng().gen_range(1..=range)
}

/// Load high scores from file
fn load_high_scores() -> io::Result<Vec<HighScore>> {
    match fs::read_to_string(HIGHSCORE_FILE) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Save new high score
fn save_score(score: HighScore) -> io::Result<()> {
    let mut scores = load_high_scores()?;
    scores.push(score);
    
    // Sort by attempts and time
    scores.sort_by(|a, b| {
        a.attempts.cmp(&b.attempts)
            .then(a.seconds.partial_cmp(&b.seconds).unwrap())
    });
    
    // Keep only top 5 scores
    scores.truncate(5);
    
    // Save to file
    let json = serde_json::to_string_pretty(&scores)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(HIGHSCORE_FILE, json)
}

/// Display high scores
fn display_high_scores() -> io::Result<()> {
    let scores = load_high_scores()?;
    if scores.is_empty() {
        println!("No high scores yet!");
        return Ok(());
    }
    
    println!("\n High Scores ");
    println!("{:<4} {:<8} {:<8} {:<10} {:<20}", 
        "Rank", "Attempts", "Time", "Difficulty", "Date");
    println!("{}", "-".repeat(50));
    
    for (i, score) in scores.iter().enumerate() {
        println!("{:<4} {:<8} {:<8.2}s {:<10} {}", 
            i + 1,
            score.attempts,
            score.seconds,
            score.difficulty,
            score.date.format("%Y-%m-%d %H:%M"));
    }
    println!();
    Ok(())
}

/// Ask if the player wants to play another round
fn ask_play_again() -> io::Result<bool> {
    loop {
        println!("\nWould you like to play again? (y/n):");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}