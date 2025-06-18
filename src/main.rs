mod app;
mod handler;
mod ui;
mod banner;
mod sound;
mod global_prefs;

use crate::app::App;
use crate::sound::SoundManager;
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

// --- CHANGE 1: Add a `Tick` variant to our event enum ---
enum AppEvent {
    Terminal(CEvent),
    Server(ServerMessage),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    global_prefs::init_global_prefs();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    let (mut server_writer, mut server_reader) = framed.split();

    let (to_server_tx, mut to_server_rx) = mpsc::unbounded_channel::<ClientMessage>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    // Task: Forward messages from our app TO the server
    let notify_disconnect_tx = event_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = to_server_rx.recv().await {
            let bytes = bincode::serialize(&msg).unwrap();
            if server_writer.send(bytes.into()).await.is_err() {
                let _ = notify_disconnect_tx.send(AppEvent::Terminal(CEvent::Resize(0,0))); // dummy event to wake main loop
                break;
            }
        }
    });

    // Task: Read messages FROM the server and send them as events
    let server_event_tx = event_tx.clone();
    let notify_disconnect_tx2 = event_tx.clone();
    tokio::spawn(async move {
        while let Some(result) = server_reader.next().await {
            match result {
                Ok(data) => {
                    if let Ok(msg) = bincode::deserialize::<ServerMessage>(&data) {
                        if server_event_tx.send(AppEvent::Server(msg)).is_err() { break; }
                    }
                }
                Err(_) => {
                    let _ = notify_disconnect_tx2.send(AppEvent::Terminal(CEvent::Resize(0,0))); // dummy event
                    break;
                },
            }
        }
    });

    // Task: Read terminal keyboard events and send them as events
    let terminal_event_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            // This now blocks until a key event occurs
            if let Ok(event) = event::read() {
                if terminal_event_tx.send(AppEvent::Terminal(event)).is_err() { break; }
            }
        }
    });

    // --- CHANGE 2: Add a new dedicated "Ticker" task ---
    // This task's ONLY job is to send a Tick event at a fixed interval.
    let tick_event_tx = event_tx;
    tokio::spawn(async move {
        let frame_rate = Duration::from_millis(100);
        loop {
            tokio::time::sleep(frame_rate).await;
            if tick_event_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });


    // --- CHANGE 3: The Main Loop now handles Ticks ---
    let sound_manager = SoundManager::new();
    let mut app = App::new(to_server_tx, &sound_manager);
    let mut server_disconnected = false;
    // Use a `while let` loop to continuously process events from the channel
    while let Some(event) = event_rx.recv().await {
        // First, handle any logic based on the event
        match event {
            AppEvent::Server(msg) => {
                app.handle_server_message(msg);
            }
            AppEvent::Terminal(CEvent::Key(key)) => {
                handler::handle_key_event(key, &mut app);
            }
            AppEvent::Terminal(CEvent::Resize(_, _)) => {
                if !server_disconnected {
                    app.set_notification("Server disconnected", None, true);
                    server_disconnected = true;
                }
            }
            // On every tick, we update the app's internal tick counter
            AppEvent::Tick => {
                app.on_tick();
            }
            _ => {}
        }
        
        // After handling logic, check if we need to quit
        if app.should_quit {
            break;
        }

        // After every event (including Ticks), we redraw the UI.
        // This ensures the animation is constantly updated.
        terminal.draw(|f| ui::ui(f, &mut app))?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}