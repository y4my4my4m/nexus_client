mod app;
mod handlers;
mod services;
mod state;
mod ui;
mod sound;
mod global_prefs;
mod model;
mod desktop_notifications;

use app::App;
use sound::SoundManager;
use common::{ClientMessage, ServerMessage};
use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{SinkExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, error::Error, io, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// Application events
enum AppEvent {
    Terminal(CEvent),
    Server(ServerMessage),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize global preferences
    global_prefs::init_global_prefs();
    
    // Enable terminal raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create event channels
    let (tx_to_server, mut rx_from_ui) = mpsc::unbounded_channel::<ClientMessage>();
    let (tx_to_ui, mut rx_from_server) = mpsc::unbounded_channel::<ServerMessage>();

    // Initialize sound manager
    let sound_manager = SoundManager::new();

    // Create app instance
    let mut app = App::new(tx_to_server, &sound_manager);

    // Get server address from command line or use default
    let server_addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    
    // Connect to server
    let stream = TcpStream::connect(&server_addr).await?;
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    // Create event loop channels
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // Spawn terminal event handler
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            
            // Check for terminal events (non-blocking)
            if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    if event_tx_clone.send(AppEvent::Terminal(event)).is_err() {
                        break;
                    }
                }
            }
            
            // Send tick event
            if event_tx_clone.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Spawn server message handler
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx_from_server.recv().await {
            if event_tx_clone.send(AppEvent::Server(msg)).is_err() {
                break;
            }
        }
    });

    // Spawn server communication handler
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle outgoing messages to server
                msg = rx_from_ui.recv() => {
                    if let Some(msg) = msg {
                        let serialized = bincode::serialize(&msg).unwrap();
                        if framed.send(serialized.into()).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Handle incoming messages from server
                result = framed.next() => {
                    match result {
                        Some(Ok(bytes)) => {
                            if let Ok(msg) = bincode::deserialize::<ServerMessage>(&bytes) {
                                if tx_to_ui.send(msg).is_err() {
                                    break;
                                }
                            }
                        }
                        Some(Err(_)) | None => {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Main application loop
    while !app.ui.should_quit {
        // Render UI
        terminal.draw(|f| ui::ui(f, &mut app))?;

        // Handle events
        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Terminal(terminal_event) => {
                    if let CEvent::Key(key) = terminal_event {
                        handlers::handle_key_event(key, &mut app);
                    }
                }
                AppEvent::Server(server_msg) => {
                    app.handle_server_message(server_msg);
                }
                AppEvent::Tick => {
                    app.on_tick();
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}