use actix_web::{get, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use regex::Regex;

use crate::components::navbar::navbar;

// Configuration constants
const MAX_SUGGESTIONS: usize = 10;

#[derive(Deserialize, Clone)]
struct GuessData {
    word: String,
    feedback: String,
}

#[derive(Deserialize)]
struct WordleRequest {
    guesses: Vec<GuessData>,
}

#[derive(Serialize)]
struct WordleResponse {
    success: bool,
    message: String,
    suggestions: Option<Vec<String>>,
}

// Validate inputs
fn validate_wordle_input(guesses: &[GuessData]) -> Result<(), String> {
    // Empty guesses are allowed - user wants initial suggestions
    if guesses.is_empty() {
        return Ok(());
    }

    for (i, guess_data) in guesses.iter().enumerate() {
        let word = &guess_data.word;
        let feedback = &guess_data.feedback;

        // Word must be exactly 5 characters
        if word.len() != 5 {
            return Err(format!("Guess {} must be exactly 5 characters", i + 1));
        }

        // Word should only contain letters
        let word_regex = Regex::new(r"^[a-z]{5}$").unwrap();
        if !word_regex.is_match(word) {
            return Err(format!("Guess {} can only contain letters", i + 1));
        }

        // Feedback must be exactly 5 characters
        if feedback.len() != 5 {
            return Err(format!("Feedback for guess {} must be exactly 5 characters", i + 1));
        }

        // Feedback should only contain 0, 1, 2
        let feedback_regex = Regex::new(r"^[012]{5}$").unwrap();
        if !feedback_regex.is_match(feedback) {
            return Err(format!("Feedback for guess {} can only contain 0 (grey), 1 (yellow), 2 (green)", i + 1));
        }
    }

    Ok(())
}

#[get("/wordle")]
pub async fn render() -> Result<HttpResponse> {
    let html_content = format!("
        <html lang=\"en\">
            <head>
                <meta charset=\"utf-8\" />
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/index.css\">
                <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">
                <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>
                <link href=\"https://fonts.googleapis.com/css2?family=Open+Sans:wght@300;400&family=Reenie+Beanie&family=Source+Code+Pro&display=swap\" rel=\"stylesheet\">
            </head>
            <body>
                {}

                <link type=\"text/css\" rel=\"stylesheet\" href=\"/static/css/wordle_solver.css\">

                <div class=\"content\">
                    <div class=\"intro\">
                        <h2 class=\"title\">🎯 Wordle <span style=\"color: #5dfa5c;\">Solver</span> 🧩</h2>
                        <p>
                            Enter your Wordle guesses and set the feedback colors. Click on letters to cycle through 
                            Grey → Yellow → Green. Add more rows as needed for your game progress.
                        </p>
                    </div>

                    <div class=\"solver-form\">
                        <div class=\"form-container\">
                            <div class=\"wordle-grid\" id=\"wordle-grid\">
                                <!-- Initial row will be added by JavaScript -->
                            </div>

                            <div class=\"controls\">
                                <button id=\"add-row-btn\" class=\"add-row-button\" title=\"Add another guess row\">
                                    ➕ Add Row
                                </button>

                                <button id=\"solve-btn\" class=\"solve-button\">
                                    <span class=\"button-text\">🔍 Get Suggestions</span>
                                    <span class=\"loading-spinner\" style=\"display: none;\">
                                        Analyzing...
                                        <span class=\"loading-dots\"></span>
                                    </span>
                                </button>
                            </div>
                        </div>
                    </div>

                    <div id=\"result-container\" class=\"result-container\" style=\"display: none;\">
                        <div id=\"result-content\" class=\"result-content\">
                        </div>
                    </div>

                    <div class=\"info-section\">
                        <h3 class=\"info-title\">How to use</h3>
                        <div class=\"instructions\">
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">⌨️</span>
                                <div>
                                    <strong>Enter Words:</strong> Type your Wordle guesses in the grid. Each row represents one guess.
                                </div>
                            </div>
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">🎨</span>
                                <div>
                                    <strong>Set Colors:</strong> Click on each letter to cycle through colors:
                                    <span class=\"color-example grey\">Grey</span> (not in word) → 
                                    <span class=\"color-example yellow\">Yellow</span> (wrong position) → 
                                    <span class=\"color-example green\">Green</span> (correct position)
                                </div>
                            </div>
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">➕</span>
                                <div>
                                    <strong>Add Rows:</strong> Click \"Add Row\" to add more guesses as you play through your Wordle game.
                                </div>
                            </div>
                            <div class=\"instruction-item\">
                                <span class=\"instruction-icon\">🧠</span>
                                <div>
                                    <strong>Get Help:</strong> Click \"Get Suggestions\" to see the best next words based on your current game state.
                                </div>
                            </div>
                        </div>

                        <div class=\"color-legend\">
                            <h4>Color Legend:</h4>
                            <div class=\"legend-items\">
                                <div class=\"legend-item\">
                                    <div class=\"legend-tile grey\"></div>
                                    <span>Grey - Letter not in word</span>
                                </div>
                                <div class=\"legend-item\">
                                    <div class=\"legend-tile yellow\"></div>
                                    <span>Yellow - Letter in word, wrong position</span>
                                </div>
                                <div class=\"legend-item\">
                                    <div class=\"legend-tile green\"></div>
                                    <span>Green - Letter in word, correct position</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class=\"trivia-section\">
                        <h3 class=\"trivia-title\">🧠 Solver Trivia</h3>
                        <div class=\"trivia-content\">
                            <p>
                                Although the words <strong>raise</strong>, <strong>slate</strong>, <strong>crate</strong>, <strong>irate</strong>, <strong>trace</strong>, <strong>arise</strong>, <strong>stare</strong>, <strong>snare</strong>, <strong>arose</strong> and <strong>least</strong> are the suggested starting words when considering entropy, in the worst case scenario you may need all 6 guesses to find the correct word using the solver.
                            </p>
                            <p>
                                The words <strong>react</strong>, <strong>roast</strong>, <strong>alien</strong>, <strong>trail</strong>, <strong>snore</strong>, <strong>train</strong>, <strong>renal</strong>, <strong>rinse</strong>, <strong>solar</strong> and <strong>sonar</strong> have slightly worse starting entropy <strong>BUT</strong> you are guaranteed to always find the correct word in at most 5 steps using this solver and one of those words as starting word.
                            </p>
                        </div>
                    </div>
                </div>

                <script type=\"text/javascript\" src=\"/static/js/wordle_solver.js\"></script>
            </body>
        </html>
    ", navbar());

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}

#[post("/api/wordle")]
pub async fn solve_wordle(data: web::Json<WordleRequest>) -> Result<HttpResponse> {
    let guesses = &data.guesses;

    // Validate inputs
    if let Err(validation_error) = validate_wordle_input(guesses) {
        return Ok(HttpResponse::BadRequest().json(WordleResponse {
            success: false,
            message: validation_error,
            suggestions: None,
        }));
    }

    // Build command - if no guesses, just call without arguments for initial suggestions
    let mut command = Command::new("./bin/wordle");
    
    if !guesses.is_empty() {
        // Build command arguments: word1 feedback1 word2 feedback2 ...
        let mut args = Vec::new();
        for guess_data in guesses {
            args.push(guess_data.word.clone());
            args.push(guess_data.feedback.clone());
        }
        command.args(&args);
    }

    // Call the C++ binary
    let output = command.output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            // If there's stderr, it's likely an error
            if !stderr.is_empty() {
                return Ok(HttpResponse::Ok().json(WordleResponse {
                    success: false,
                    message: format!("Solver error: {}", stderr),
                    suggestions: None,
                }));
            }

            // Parse the output - each line has "entropy word"
            let suggestions: Vec<String> = stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(MAX_SUGGESTIONS) // Limit to top 20 suggestions
                .map(|line| {
                    // Extract just the word part (skip entropy)
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].to_uppercase()
                    } else {
                        line.trim().to_uppercase()
                    }
                })
                .collect();

            if suggestions.is_empty() {
                return Ok(HttpResponse::Ok().json(WordleResponse {
                    success: false,
                    message: "No words found matching your criteria. Try adjusting your guesses.".to_string(),
                    suggestions: None,
                }));
            }

            let message = if guesses.is_empty() {
                format!("Best {} starting word{}", suggestions.len(), if suggestions.len() == 1 { "" } else { "s" })
            } else {
                format!("{} next best guesses (from best to worst)", suggestions.len())
            };

            Ok(HttpResponse::Ok().json(WordleResponse {
                success: true,
                message,
                suggestions: Some(suggestions),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(WordleResponse {
                success: false,
                message: format!("Failed to execute Wordle solver: {}. Make sure the wordle binary is available in the current directory.", e),
                suggestions: None,
            }))
        }
    }
} 