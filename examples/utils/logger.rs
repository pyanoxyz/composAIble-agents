use env_logger::{ Builder, fmt::Color };
use std::io::Write;
use colored::*;

fn setup_color_logger() {
    Builder::from_default_env()
        .format(|buf, record| {
            match record.level() {
                log::Level::Info => {
                    writeln!(buf, "{} {}", "ℹ️".blue(), record.args().to_string().bright_white())
                }
                log::Level::Error => {
                    writeln!(buf, "{} {}", "✖️".red(), record.args().to_string().bright_red())
                }
                log::Level::Warn => {
                    writeln!(buf, "{} {}", "⚠️".yellow(), record.args().to_string().bright_yellow())
                }
                log::Level::Debug => {
                    writeln!(buf, "{} {}", "🔧".cyan(), record.args().to_string().bright_cyan())
                }
                log::Level::Trace => {
                    writeln!(
                        buf,
                        "{} {}",
                        "👉".magenta(),
                        record.args().to_string().bright_magenta()
                    )
                }
            }
        })
        .init();
}

// In your initialization code
pub fn setup_logger() {
    Builder::from_default_env()
        .format(|buf, record| {
            if record.level() == log::Level::Info {
                writeln!(buf, "{}", record.args())
            } else {
                // Keep full format for other log levels
                writeln!(buf, "[{}] {}: {}", record.level(), record.target(), record.args())
            }
        })
        .init();
}
