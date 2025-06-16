mod app;
mod handler;
mod ui;

use crate::app::App;
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

enum AppEvent {
    Terminal(CEvent),
    Server(ServerMessage),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // --- NO MORE USERNAME PROMPT HERE ---

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    let (mut server_writer, mut server_reader) = framed.split();

    let (to_server_tx, mut to_server_rx) = mpsc::unbounded_channel::<ClientMessage>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // --- FIX: NO INITIAL HANDSHAKE ---
    // The login/register screens handle this now.

    tokio::spawn(async move {
        while let Some(msg) = to_server_rx.recv().await {
            let bytes = bincode::serialize(&msg).unwrap();
            if server_writer.send(bytes.into()).await.is_err() {
                eprintln!("Failed to send message to server. Connection closed.");
                break;
            }
        }
    });

    let server_event_tx = event_tx.clone();
    tokio::spawn(async move {
        while let Some(result) = server_reader.next().await {
            match result {
                Ok(data) => {
                    if let Ok(msg) = bincode::deserialize::<ServerMessage>(&data) {
                        if server_event_tx.send(AppEvent::Server(msg)).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    let terminal_event_tx = event_tx;
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

    let mut app = App::new(to_server_tx);
    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Server(msg) => {
                    app.handle_server_message(msg);
                }
                AppEvent::Terminal(CEvent::Key(key)) => {
                    handler::handle_key_event(key, &mut app);
                }
                _ => {}
            }
        }

        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}