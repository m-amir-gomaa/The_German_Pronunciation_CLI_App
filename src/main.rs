mod config;
mod input;
mod tts;
mod ipa;
mod stt;
mod scoring;
mod db;
mod autocomplete;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;

#[derive(Parser)]
#[command(
    name = "gp",
    about = "German Pronunciation CLI — IPA, TTS, Vim motions, pronunciation scoring",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Pronounce a German word or phrase (shows IPA + plays audio)
    Pronounce {
        /// The German word or phrase to pronounce
        word: String,
    },

    /// Interactive pronunciation practice with mic scoring
    Practice {
        /// Word to practice (optional — picks from saved list if omitted)
        word: Option<String>,
    },

    /// Save a word to your personal word list
    Save {
        /// German word to save
        word: String,
    },

    /// List your saved words and scores
    List,

    /// Flashcard drill mode — cycles through your saved word list
    Drill,

    /// Launch interactive REPL (Vim motions + Tab autocomplete)
    Repl,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = Config::load()?;
    let db = db::Db::open()?;

    match cli.command {
        Command::Pronounce { word } => {
            let ipa = ipa::get_ipa(&word)?;
            println!("IPA: /{}/", ipa);
            tts::speak(&word, &config).await?;
        }

        Command::Practice { word } => {
            let target = match word {
                Some(w) => w,
                None => db.random_word()?,
            };
            let ipa = ipa::get_ipa(&target)?;
            println!("Word:  {}", target);
            println!("IPA:   /{}/", ipa);
            println!("Listening… (speak now)");
            tts::speak(&target, &config).await?;

            let heard = stt::record_and_transcribe(&config)?;
            let score = scoring::score(&target, &heard);
            println!("You said:  \"{}\"", heard);
            println!("Score:     {}%", score);

            db.record_attempt(&target, score)?;
        }

        Command::Save { word } => {
            let ipa = ipa::get_ipa(&word)?;
            db.save_word(&word, &ipa)?;
            println!("Saved: {} /{}/", word, ipa);
        }

        Command::List => {
            let words = db.list_words()?;
            if words.is_empty() {
                println!("No saved words yet. Use `gp save <word>` to add one.");
            } else {
                println!("{:<25} {:<30} {}", "Word", "IPA", "Best Score");
                println!("{}", "-".repeat(65));
                for (word, ipa, score) in words {
                    println!("{:<25} {:<30} {}%", word, ipa, score.unwrap_or(0));
                }
            }
        }

        Command::Drill => {
            println!("Starting drill mode — press Ctrl+C to stop.\n");
            loop {
                let word = db.random_word()?;
                let ipa = ipa::get_ipa(&word)?;
                println!("─────────────────────────────");
                println!("Word:  {}", word);
                println!("IPA:   /{}/", ipa);
                tts::speak(&word, &config).await?;
                println!("Your turn…");
                let heard = stt::record_and_transcribe(&config)?;
                let score = scoring::score(&word, &heard);
                println!("You said: \"{}\" — {}%", heard, score);
                db.record_attempt(&word, score)?;
                println!();
            }
        }

        Command::Repl => {
            println!("German Pronouncer REPL — type a word and press Enter.");
            if config.editor.vim_mode {
                println!("Vim mode: ON  (i → Insert, Esc → Normal, Tab → autocomplete)");
            }
            input::repl::run(&config, &db).await?;
        }
    }

    Ok(())
}
