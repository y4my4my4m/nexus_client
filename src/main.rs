mod app;
mod handler;
mod ui;

use crate::app::App;
use common::{ClientMessage, ServerMessage};
use crossterm::{
    event::{self, Event as CEvent}, // Remove unused KeyCode
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{SinkExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, error::Error, io, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

// Define an enum for events our app will handle
enum AppEvent {
    Terminal(CEvent),
    Server(ServerMessage),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // --- Get Username --- (same as before)
    let mut username = String::new();
    println!("Please enter your username:");
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    // --- Setup Terminal --- (same as before)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- Network Setup ---
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    // *** FIX: SPLIT THE FRAMED STREAM ***
    let (mut server_writer, mut server_reader) = framed.split();

    // --- Channel for sending messages TO the server ---
    let (to_server_tx, mut to_server_rx) = mpsc::unbounded_channel::<ClientMessage>();

    // --- Channel for receiving events FROM other tasks ---
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // --- Initial Handshake ---
    to_server_tx.send(ClientMessage::SetUsername(username.clone()))?;
    to_server_tx.send(ClientMessage::GetForums)?;

    // --- Task: Forward messages from our app TO the server ---
    tokio::spawn(async move {
        while let Some(msg) = to_server_rx.recv().await {
            let bytes = bincode::serialize(&msg).unwrap();
            if server_writer.send(bytes.into()).await.is_err() {
                eprintln!("Failed to send message to server. Connection closed.");
                break;
            }
        }
    });

    // --- Task: Read messages FROM the server and send them as events ---
    let server_event_tx = event_tx.clone();
    tokio::spawn(async move {
        while let Some(result) = server_reader.next().await {
            match result {
                Ok(data) => {
                    if let Ok(msg) = bincode::deserialize::<ServerMessage>(&data) {
                        if server_event_tx.send(AppEvent::Server(msg)).is_err() {
                            break; // Channel closed
                        }
                    }
                }
                Err(_) => break, // Connection error
            }
        }
    });

    // --- Task: Read terminal events and send them as events ---
    let terminal_event_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    if terminal_event_tx.send(AppEvent::Terminal(event)).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // --- Main Loop: Owns `app` and handles events ---
    let mut app = App::new(username, to_server_tx.clone());
    loop {
        // Draw the UI
        terminal.draw(|f| ui::ui(f, &mut app))?;

        // Wait for the next event from any source
        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Server(msg) => {
                    app.handle_server_message(msg);
                }
                AppEvent::Terminal(CEvent::Key(key)) => {
                    handler::handle_key_event(key, &mut app);
                }
                _ => {} // Ignore other terminal events like mouse for now
            }
        }

        // On every loop, we can tick the app
        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    // --- Cleanup ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}