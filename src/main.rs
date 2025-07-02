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
use nexus_tui_common::{ClientMessage, ServerMessage};
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
use tokio_rustls::rustls::{self, ClientConfig as RustlsClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use rustls_pemfile;
use tokio_rustls::rustls::client::danger::ServerCertVerifier;

fn load_root_cert(path: &str) -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    let certfile = File::open(path).expect("Cannot open cert.pem");
    let mut reader = BufReader::new(certfile);
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader).filter_map(|res| res.ok()).collect();
    for cert in certs {
        root_store.add(cert).unwrap();
    }
    root_store
}

fn system_root_store() -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs()
        .expect("could not load platform certs");
    for cert in certs {
        root_store.add(cert).unwrap();
    }
    root_store
}

/// Application events
enum AppEvent {
    Terminal(CEvent),
    Server(ServerMessage),
    Tick,
    RetryConnection, // New event for connection retry
    ConnectionLost, // New event for when connection is lost
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
    let cert_path = env::args().nth(2); // Optional cert path
    let parts: Vec<String> = server_addr.split(':').map(|s| s.to_string()).collect();
    let server_host = parts.get(0).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
    let server_port = parts.get(1).cloned().unwrap_or_else(|| "8080".to_string());

    // TLS setup
    let root_store = if let Some(path) = cert_path {
        load_root_cert(&path)
    } else {
        system_root_store()
    };
    let tls_config = RustlsClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(tls_config));
    let server_name = ServerName::try_from(server_host.clone()).unwrap();

    // Try to connect to server with error handling (TLS)
    let tcp_stream = TcpStream::connect(&server_addr).await;
    let connection_result = match tcp_stream {
        Ok(stream) => {
            match tls_connector.connect(server_name.clone(), stream).await {
                Ok(tls_stream) => Ok(tls_stream),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("TLS error: {}", e))),
            }
        },
        Err(e) => Err(e),
    };
    
    // Show error popup if initial connection fails
    if let Err(e) = &connection_result {
        let error_msg = match e.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                format!("Connection refused to {}", server_addr)
            }
            std::io::ErrorKind::TimedOut => {
                format!("Connection timeout to {}", server_addr)
            }
            std::io::ErrorKind::NotFound => {
                format!("Host not found: {}", server_addr)
            }
            _ => {
                format!("Network error: {}", e)
            }
        };
        
        app.ui.show_server_error(error_msg);
        app.sound_manager.play(sound::SoundType::Error);
    }

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

    // Server communication handler (only if initially connected)
    let mut server_comm_handle = None;
    if connection_result.is_ok() {
        let stream = connection_result.unwrap();
        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        
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
        let event_tx_clone = event_tx.clone();
        server_comm_handle = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle outgoing messages to server
                    msg = rx_from_ui.recv() => {
                        if let Some(msg) = msg {
                            let serialized = bincode::serialize(&msg).unwrap();
                            if framed.send(serialized.into()).await.is_err() {
                                // Connection lost while sending
                                let _ = event_tx_clone.send(AppEvent::ConnectionLost);
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
                                // Connection lost while receiving
                                let _ = event_tx_clone.send(AppEvent::ConnectionLost);
                                break;
                            }
                        }
                    }
                }
            }
        }));
    }

    // Main application loop
    while !app.ui.should_quit {
        // Check for retry connection request
        if app.ui.should_retry_connection {
            app.ui.should_retry_connection = false;
            // Attempt to reconnect (TLS)
            match TcpStream::connect(&server_addr).await {
                Ok(stream) => {
                    match tls_connector.connect(server_name.clone(), stream).await {
                        Ok(tls_stream) => {
                            app.sound_manager.play(sound::SoundType::LoginSuccess);
                            if let Some(handle) = server_comm_handle.take() {
                                handle.abort();
                            }
                            let (new_tx_to_server, mut new_rx_from_ui) = mpsc::unbounded_channel::<ClientMessage>();
                            let (new_tx_to_ui, mut new_rx_from_server) = mpsc::unbounded_channel::<ServerMessage>();
                            app.to_server = new_tx_to_server;
                            let mut framed = Framed::new(tls_stream, LengthDelimitedCodec::new());
                            // Spawn new server message handler
                            let event_tx_clone = event_tx.clone();
                            tokio::spawn(async move {
                                while let Some(msg) = new_rx_from_server.recv().await {
                                    if event_tx_clone.send(AppEvent::Server(msg)).is_err() {
                                        break;
                                    }
                                }
                            });

                            // Spawn new server communication handler
                            let event_tx_clone = event_tx.clone();
                            server_comm_handle = Some(tokio::spawn(async move {
                                loop {
                                    tokio::select! {
                                        msg = new_rx_from_ui.recv() => {
                                            if let Some(msg) = msg {
                                                let serialized = bincode::serialize(&msg).unwrap();
                                                if framed.send(serialized.into()).await.is_err() {
                                                    // Connection lost while sending
                                                    let _ = event_tx_clone.send(AppEvent::ConnectionLost);
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        
                                        result = framed.next() => {
                                            match result {
                                                Some(Ok(bytes)) => {
                                                    if let Ok(msg) = bincode::deserialize::<ServerMessage>(&bytes) {
                                                        if new_tx_to_ui.send(msg).is_err() {
                                                            break;
                                                        }
                                                    }
                                                }
                                                Some(Err(_)) | None => {
                                                    // Connection lost while receiving
                                                    let _ = event_tx_clone.send(AppEvent::ConnectionLost);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }));
                        }
                        Err(e) => {
                            let error_msg = format!("TLS error: {}", e);
                            app.ui.show_server_error(error_msg);
                            app.sound_manager.play(sound::SoundType::Error);
                        }
                    }
                }
                Err(e) => {
                    // Connection failed, show error and continue
                    let error_msg = match e.kind() {
                        std::io::ErrorKind::ConnectionRefused => {
                            format!("Connection refused to {}", server_addr)
                        }
                        std::io::ErrorKind::TimedOut => {
                            format!("Connection timeout to {}", server_addr)
                        }
                        std::io::ErrorKind::NotFound => {
                            format!("Host not found: {}", server_addr)
                        }
                        _ => {
                            format!("Network error: {}", e)
                        }
                    };
                    
                    app.ui.show_server_error(error_msg);
                    app.sound_manager.play(sound::SoundType::Error);
                }
            }
        }

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
                AppEvent::RetryConnection => {
                    app.ui.should_retry_connection = true;
                }
                AppEvent::ConnectionLost => {
                    // Handle connection lost event (e.g., show a message, play a sound, etc.)
                    app.ui.show_server_error("Connection to server was lost.".to_string());
                    app.sound_manager.play(sound::SoundType::Error);
                }
            }
        }
    }

    // Cleanup
    if let Some(handle) = server_comm_handle {
        handle.abort();
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}