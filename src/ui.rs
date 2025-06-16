use crate::app::{App, AppMode};
use rand::prelude::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

const BANNER: &str = r"
██████╗ ██╗   ██╗██████╗ ███████╗██████╗  ██████╗ ██████╗ ███████╗
██╔══██╗╚██╗ ██╔╝██╔══██╗██╔════╝██╔══██╗██╔═══██╗██╔══██╗██╔════╝
██████╔╝ ╚████╔╝ ██████╔╝█████╗  ██████╔╝██║   ██║██████╔╝███████╗
██╔═══╝   ╚██╔╝  ██╔══██╗██╔══╝  ██╔══██╗██║   ██║██╔══██╗╚════██║
██║        ██║   ██║  ██║███████╗██║  ██║╚██████╔╝██║  ██║███████║
╚═╝        ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
";
const MOTD: &str =
    "Welcome to the Nexus Point BBS. Stay low, stay fast. The corps are watching. Post wisely.";
const HELP_TEXT: &str = "| [Q]uit | [↑↓] Navigate | [Enter] Select | [Esc] Back |";

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(70),
                Constraint::Min(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // Glitchy Banner
    let banner_text = glitch_text(BANNER, app.tick_count);
    let banner = Paragraph::new(banner_text)
        .style(Style::default().fg(Color::Magenta))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Double),
        );
    f.render_widget(banner, chunks[0]);

    // Main content based on mode
    match app.mode {
        AppMode::MainMenu => draw_main_menu(f, app, chunks[1]),
        AppMode::ForumList => draw_forum_list(f, app, chunks[1]),
        AppMode::ThreadList => draw_thread_list(f, app, chunks[1]),
        AppMode::PostView => draw_post_view(f, app, chunks[1]),
        AppMode::Chat => draw_chat(f, app, chunks[1]),
    }

    // Footer
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_type(BorderType::Double)
        .title(" Status ");
    let footer = Paragraph::new(HELP_TEXT)
        .block(footer_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(footer, chunks[2]);
}

fn draw_main_menu(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let menu_items = vec![
        ListItem::new(Line::from(Span::styled(
            ">> Forums",
            Style::default().fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(Span::styled(
            ">> Chat",
            Style::default().fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(Span::styled(
            ">> Log Off",
            Style::default().fg(Color::Red),
        ))),
    ];

    let menu_list = List::new(menu_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Main Menu")
                .border_type(BorderType::Double),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    f.render_stateful_widget(menu_list, chunks[0], &mut app.main_menu_state);

    let motd_block = Block::default()
        .borders(Borders::ALL)
        .title("Message of the Day")
        .border_type(BorderType::Double);
    let motd_paragraph = Paragraph::new(MOTD)
        .wrap(Wrap { trim: true })
        .block(motd_block)
        .style(Style::default().fg(Color::Green));
    f.render_widget(motd_paragraph, chunks[1]);
}

fn draw_forum_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .forums
        .iter()
        .map(|forum| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<25}", forum.name), Style::default().fg(Color::Cyan)),
                Span::raw(forum.description.clone()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Forums")
                .border_type(BorderType::Double),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.forum_list_state);
}

fn draw_thread_list(f: &mut Frame, app: &mut App, area: Rect) {
    let forum = app.forums.get(app.current_forum_index.unwrap()).unwrap();
    let items: Vec<ListItem> = forum
        .threads
        .iter()
        .map(|thread| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<40}", thread.title),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(format!("by {}", thread.author)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Threads in '{}'", forum.name))
                .border_type(BorderType::Double),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.thread_list_state);
}

fn draw_post_view(f: &mut Frame, app: &mut App, area: Rect) {
    let forum = app.forums.get(app.current_forum_index.unwrap()).unwrap();
    let thread = forum.threads.get(app.current_thread_index.unwrap()).unwrap();

    let mut text: Vec<Line> = Vec::new();
    for post in &thread.posts {
        text.push(Line::from(vec![Span::styled(
            format!("From: {} ", post.author),
            Style::default().fg(Color::Yellow),
        )]));
        text.push(Line::from(Span::raw("---------------------------------")));
        text.push(Line::from(Span::raw(&post.content)));
        text.push(Line::from(Span::raw(""))); // Spacer
    }

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Reading: {}", thread.title))
                .border_type(BorderType::Double),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_chat(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    let messages: Vec<ListItem> = app
        .chat_messages
        .iter()
        .rev()
        .take(chunks[0].height as usize) // Take only as many messages as can fit
        .rev() // reverse it back to show oldest at the top
        .map(|msg| {
            let content = Line::from(vec![
                Span::styled(
                    format!("<{}>: ", msg.author),
                    Style::default().fg(msg.color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(msg.content.clone()),
            ]);
            ListItem::new(content)
        })
        .collect();

    let messages_list = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Live Chat // #general")
            .border_type(BorderType::Double),
    );

    f.render_widget(messages_list, chunks[0]);

    let input = Paragraph::new(app.chat_input.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_type(BorderType::Double),
        );

    f.render_widget(input, chunks[1]);
    f.set_cursor(
        chunks[1].x + app.chat_input.len() as u16 + 1,
        chunks[1].y + 1,
    );
}

fn glitch_text(text: &str, tick_count: u64) -> String {
    let mut rng = rand::thread_rng();
    text.chars()
        .map(|c| {
            if c.is_whitespace() || c == '╗' || c == '╔' || c == '╝' || c == '╚' || c == '═' || c == '║' {
                c
            } else if rng.gen_bool(0.01 + (tick_count as f64 * 0.001).sin().powi(2) * 0.05) {
                ['█', '▓', '▒', '░', '>', '!', '?', '#', '$']
                    .choose(&mut rng)
                    .unwrap_or(&c)
                    .clone()
            } else {
                c
            }
        })
        .collect()
}