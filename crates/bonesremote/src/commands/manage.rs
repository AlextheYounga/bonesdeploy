use std::io::{self, Stdout};
use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::config;
use crate::release_state;

const MENU_ITEMS: [MenuItem; 3] = [
    MenuItem {
        title: "Releases",
        description: "View staged/current/previous releases and prepare rollbacks.",
        page: Page::Releases,
    },
    MenuItem {
        title: "Site",
        description: "Inspect Nginx and SSL state, plus active site config status.",
        page: Page::Site,
    },
    MenuItem {
        title: "Traffic",
        description: "Review request trends and endpoint activity from GoAccess data.",
        page: Page::Traffic,
    },
];

const LOGO_LINES: [&str; 7] = [
    r"             _.--------._              ",
    r"          .-'   .--.    '-.           ",
    r"        .'     (o  o)      '.         ",
    r"       /   [>$]  /\   _      \        ",
    r"      |      .-./__\.' '.     |       ",
    r"      |_____/___\__/\___\_____|       ",
    r"          /_/\_/      \_/\_\          ",
];

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;

    let mut app = App::new(cfg.data.project_name.clone(), cfg.data.host.clone());
    let mut terminal = setup_terminal()?;

    let result = run_loop(&mut terminal, &cfg, &mut app);
    let cleanup_result = teardown_terminal(&mut terminal);

    result.and(cleanup_result)
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, cfg: &config::BonesConfig, app: &mut App) -> Result<()> {
    app.refresh_releases(cfg);

    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if !event::poll(Duration::from_millis(150)).context("Failed to poll terminal events")? {
            continue;
        }

        let input = event::read().context("Failed to read terminal event")?;
        let Event::Key(key) = input else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match app.page {
            Page::Home => handle_home_key(app, key.code),
            Page::Releases => handle_releases_key(app, cfg, key.code),
            Page::Site | Page::Traffic => handle_basic_page_key(app, key.code),
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_home_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Up => {
            app.menu_index = if app.menu_index == 0 { MENU_ITEMS.len() - 1 } else { app.menu_index - 1 };
        }
        KeyCode::Down => {
            app.menu_index = (app.menu_index + 1) % MENU_ITEMS.len();
        }
        KeyCode::Enter => {
            app.page = MENU_ITEMS[app.menu_index].page;
            app.status = format!("Opened {}", MENU_ITEMS[app.menu_index].title);
        }
        _ => {}
    }
}

fn handle_releases_key(app: &mut App, cfg: &config::BonesConfig, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc | KeyCode::Char('b') => {
            app.page = Page::Home;
            app.status = "Back to home".to_string();
        }
        KeyCode::Char('r') => app.refresh_releases(cfg),
        _ => {}
    }
}

fn handle_basic_page_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc | KeyCode::Char('b') => {
            app.page = Page::Home;
            app.status = "Back to home".to_string();
        }
        _ => {}
    }
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    match app.page {
        Page::Home => draw_home(frame, app),
        Page::Releases => draw_releases(frame, app),
        Page::Site => draw_placeholder_page(
            frame,
            app,
            "Site",
            "Site diagnostics page is scaffolded. Next: nginx status, SSL validity, and config viewer.",
        ),
        Page::Traffic => draw_placeholder_page(
            frame,
            app,
            "Traffic",
            "Traffic page is scaffolded. Next: GoAccess summary cards and top-path metrics.",
        ),
    }
}

fn draw_home(frame: &mut ratatui::Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    let logo = Paragraph::new(LOGO_LINES.join("\n"))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(" BonesDeploy "))
        .wrap(Wrap { trim: false });
    frame.render_widget(logo, chunks[0]);

    let items: Vec<ListItem<'_>> = MENU_ITEMS
        .iter()
        .map(|item| {
            ListItem::new(vec![
                Line::from(Span::styled(item.title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(Span::raw(item.description)),
            ])
        })
        .collect();

    let menu = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ")
        .block(Block::default().borders(Borders::ALL).title(format!(" Manage {} @ {} ", app.project_name, app.host)));

    let mut list_state = ListState::default();
    list_state.select(Some(app.menu_index));
    frame.render_stateful_widget(menu, chunks[1], &mut list_state);

    draw_footer(frame, chunks[2], app, "↑/↓ move  Enter open  q quit");
}

fn draw_releases(frame: &mut ratatui::Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    let body = match &app.releases {
        Some(snapshot) => snapshot.to_text(),
        None => "Release data not loaded yet".to_string(),
    };

    let paragraph = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(format!(" Releases: {} @ {} ", app.project_name, app.host)))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, chunks[0]);

    draw_footer(frame, chunks[1], app, "r refresh  b/esc back  q quit");
}

fn draw_placeholder_page(frame: &mut ratatui::Frame<'_>, app: &App, name: &str, message: &str) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    let paragraph = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title(format!(" {name}: {} @ {} ", app.project_name, app.host)))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, chunks[0]);

    draw_footer(frame, chunks[1], app, "b/esc back  q quit");
}

fn draw_footer(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App, keys: &str) {
    let last_refresh = app
        .last_refresh
        .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or_else(|| "never".to_string(), |duration| duration.as_secs().to_string());

    let footer = Paragraph::new(format!("{keys}  |  last_refresh_unix: {last_refresh}  |  {}", app.status))
        .block(Block::default().borders(Borders::ALL).title(" Keys "))
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, area);
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("Failed to enable raw terminal mode")?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("Failed to initialize terminal")
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("Failed to disable raw terminal mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to restore terminal cursor")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Home,
    Releases,
    Site,
    Traffic,
}

struct MenuItem {
    title: &'static str,
    description: &'static str,
    page: Page,
}

#[derive(Debug)]
struct App {
    page: Page,
    menu_index: usize,
    should_quit: bool,
    project_name: String,
    host: String,
    status: String,
    last_refresh: Option<SystemTime>,
    releases: Option<ReleaseSnapshot>,
}

impl App {
    fn new(project_name: String, host: String) -> Self {
        Self {
            page: Page::Home,
            menu_index: 0,
            should_quit: false,
            project_name,
            host,
            status: "Ready".to_string(),
            last_refresh: None,
            releases: None,
        }
    }

    fn refresh_releases(&mut self, cfg: &config::BonesConfig) {
        self.releases = Some(match load_release_snapshot(cfg) {
            Ok(snapshot) => {
                self.status = "Release state refreshed".to_string();
                snapshot
            }
            Err(error) => {
                self.status = "Release refresh failed".to_string();
                ReleaseSnapshot { current: None, staged: None, releases: Vec::new(), error: Some(error.to_string()) }
            }
        });

        self.last_refresh = Some(SystemTime::now());
    }
}

#[derive(Debug)]
struct ReleaseSnapshot {
    current: Option<String>,
    staged: Option<String>,
    releases: Vec<String>,
    error: Option<String>,
}

impl ReleaseSnapshot {
    fn to_text(&self) -> String {
        if let Some(error) = &self.error {
            return format!("Failed to read release state:\n{error}");
        }

        let mut lines = vec![
            format!("Current release: {}", self.current.as_deref().unwrap_or("<none>")),
            format!("Staged release: {}", self.staged.as_deref().unwrap_or("<none>")),
            String::new(),
            "Releases (oldest -> newest):".to_string(),
        ];

        if self.releases.is_empty() {
            lines.push("  - <none>".to_string());
        } else {
            for release in &self.releases {
                let marker = if self.current.as_deref() == Some(release.as_str()) { "*" } else { " " };
                lines.push(format!("{marker} {release}"));
            }
        }

        lines.join("\n")
    }
}

fn load_release_snapshot(cfg: &config::BonesConfig) -> Result<ReleaseSnapshot> {
    Ok(ReleaseSnapshot {
        current: release_state::current_release_name(cfg).ok(),
        staged: release_state::read_staged_release(cfg).ok(),
        releases: release_state::list_releases_sorted(cfg)?,
        error: None,
    })
}
