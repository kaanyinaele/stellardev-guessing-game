use eframe::egui;
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

struct GuessingGame {
    secret_number: u32,
    current_guess: String,
    message: String,
    attempts: u32,
    difficulty: u32,
    game_won: bool,
    high_scores: Vec<HighScore>,
    start_time: std::time::Instant,
}

impl Default for GuessingGame {
    fn default() -> Self {
        Self {
            secret_number: rand::thread_rng().gen_range(1..=100),
            current_guess: String::new(),
            message: "Guess a number between 1 and 100!".to_string(),
            attempts: 0,
            difficulty: 100,
            game_won: false,
            high_scores: load_high_scores().unwrap_or_default(),
            start_time: std::time::Instant::now(),
        }
    }
}

impl GuessingGame {
    fn new_game(&mut self) {
        self.secret_number = rand::thread_rng().gen_range(1..=self.difficulty);
        self.current_guess.clear();
        self.message = format!("Guess a number between 1 and {}!", self.difficulty);
        self.attempts = 0;
        self.game_won = false;
        self.start_time = std::time::Instant::now();
    }

    fn check_guess(&mut self) {
        if let Ok(guess) = self.current_guess.parse::<u32>() {
            if !self.game_won {
                self.attempts += 1;
                
                if guess < 1 || guess > self.difficulty {
                    self.message = format!("Please enter a number between 1 and {}!", self.difficulty);
                    return;
                }

                match guess.cmp(&self.secret_number) {
                    std::cmp::Ordering::Less => self.message = "Higher!".to_string(),
                    std::cmp::Ordering::Greater => self.message = "Lower!".to_string(),
                    std::cmp::Ordering::Equal => {
                        let duration = self.start_time.elapsed();
                        self.game_won = true;
                        self.message = format!(
                            "Correct! You won in {} attempts and {:.2} seconds!",
                            self.attempts,
                            duration.as_secs_f64()
                        );
                        
                        // Save high score
                        let score = HighScore {
                            attempts: self.attempts,
                            seconds: duration.as_secs_f64(),
                            difficulty: self.difficulty,
                            date: Local::now(),
                        };
                        if let Ok(()) = save_score(score) {
                            self.high_scores = load_high_scores().unwrap_or_default();
                        }
                    }
                }
            }
        } else {
            self.message = "Please enter a valid number!".to_string();
        }
        self.current_guess.clear();
    }
}

impl eframe::App for GuessingGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Number Guessing Game");
            ui.add_space(20.0);

            // Difficulty selection
            ui.horizontal(|ui| {
                ui.label("Difficulty:");
                if ui.button("Easy (50)").clicked() && !self.game_won {
                    self.difficulty = 50;
                    self.new_game();
                }
                if ui.button("Medium (100)").clicked() && !self.game_won {
                    self.difficulty = 100;
                    self.new_game();
                }
                if ui.button("Hard (200)").clicked() && !self.game_won {
                    self.difficulty = 200;
                    self.new_game();
                }
            });

            ui.add_space(20.0);

            // Game input
            ui.horizontal(|ui| {
                let text_edit = ui.text_edit_singleline(&mut self.current_guess);
                text_edit.request_focus();
                
                if ui.button("Guess").clicked() || 
                   (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    self.check_guess();
                }
            });

            if ui.button("New Game").clicked() {
                self.new_game();
            }

            ui.add_space(10.0);
            ui.label(&self.message);
            ui.add_space(20.0);

            // High Scores
            ui.heading("High Scores");
            egui::Grid::new("high_scores")
                .num_columns(4)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Attempts");
                    ui.label("Time");
                    ui.label("Difficulty");
                    ui.label("Date");
                    ui.end_row();

                    for score in &self.high_scores {
                        ui.label(score.attempts.to_string());
                        ui.label(format!("{:.2}s", score.seconds));
                        ui.label(score.difficulty.to_string());
                        ui.label(score.date.format("%Y-%m-%d %H:%M").to_string());
                        ui.end_row();
                    }
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Guessing Game",
        options,
        Box::new(|_cc| Box::new(GuessingGame::default())),
    )
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