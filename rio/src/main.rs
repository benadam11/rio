mod crosswords;
mod event;
mod layout;
mod performer;
mod renderer;
mod sequencer;
mod term;
mod window;
use crate::event::EventP;
use crate::sequencer::Sequencer;

pub fn setup_environment_variables(_config: &config::Config) {
    let terminfo = if teletypewriter::terminfo_exists("rio") {
        "rio"
    } else {
        "xterm-256color"
    };
    std::env::set_var("TERM", terminfo);
    std::env::set_var("COLORTERM", "truecolor");
    std::env::remove_var("DESKTOP_STARTUP_ID");

    if std::env::var("SHELL").is_err() {
        std::env::set_var("TERM", "bash")
    }

    #[cfg(target_os = "macos")]
    std::env::set_current_dir(dirs::home_dir().unwrap()).unwrap();

    // Set env vars from config.
    // for (key, value) in config.env.iter() {
    //     std::env::set_var(key, value);
    // }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::load();
    setup_environment_variables(&config);

    let window_event_loop =
        winit::event_loop::EventLoopBuilder::<EventP>::with_user_event().build();
    let mut sequencer = Sequencer::new(config);
    let result = sequencer.run(window_event_loop);

    result.await
}
