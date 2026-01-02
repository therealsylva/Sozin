use crate::network::{InterfaceState, InterfaceType, NetworkInterface, NetworkManager, WirelessMode};
use crate::scanner::{signal_to_bars, WifiNetwork, WifiScanner};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};
use std::io;

/// Application state
pub struct App {
    pub running: bool,
    pub current_tab: usize,
    pub interfaces: Vec<NetworkInterface>,
    pub interface_state: ListState,
    pub networks: Vec<WifiNetwork>,
    pub network_state: TableState,
    pub selected_interface: Option<String>,
    pub status_message: String,
    pub show_help: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Rename,
    MacInput,
    ChannelInput,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_tab: 0,
            interfaces: Vec::new(),
            interface_state: ListState::default(),
            networks: Vec::new(),
            network_state: TableState::default(),
            selected_interface: None,
            status_message: String::new(),
            show_help: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.refresh_interfaces();
        app
    }

    pub fn refresh_interfaces(&mut self) {
        match NetworkManager::get_interfaces() {
            Ok(interfaces) => {
                self.interfaces = interfaces;
                if self.interface_state.selected().is_none() && !self.interfaces.is_empty() {
                    self.interface_state.select(Some(0));
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    pub fn selected_interface(&self) -> Option<&NetworkInterface> {
        self.interface_state
            .selected()
            .and_then(|i| self.interfaces.get(i))
    }

    pub fn next_interface(&mut self) {
        if self.interfaces.is_empty() {
            return;
        }
        let i = match self.interface_state.selected() {
            Some(i) => {
                if i >= self.interfaces.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.interface_state.select(Some(i));
    }

    pub fn previous_interface(&mut self) {
        if self.interfaces.is_empty() {
            return;
        }
        let i = match self.interface_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.interfaces.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.interface_state.select(Some(i));
    }

    pub fn next_network(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let i = match self.network_state.selected() {
            Some(i) => {
                if i >= self.networks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.network_state.select(Some(i));
    }

    pub fn previous_network(&mut self) {
        if self.networks.is_empty() {
            return;
        }
        let i = match self.network_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.networks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.network_state.select(Some(i));
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 2;
        } else {
            self.current_tab -= 1;
        }
    }
}

/// Run the TUI application
pub async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                app.running = false;
                                return Ok(());
                            }
                            KeyCode::Char('?') | KeyCode::F(1) => {
                                app.show_help = !app.show_help;
                            }
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.previous_tab(),
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.current_tab == 0 {
                                    app.next_interface();
                                } else if app.current_tab == 1 {
                                    app.next_network();
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.current_tab == 0 {
                                    app.previous_interface();
                                } else if app.current_tab == 1 {
                                    app.previous_network();
                                }
                            }
                            KeyCode::Char('r') => {
                                app.refresh_interfaces();
                                app.status_message = "Interfaces refreshed".to_string();
                            }
                            KeyCode::Char('m') => {
                                // Toggle monitor mode
                                if let Some(iface) = app.selected_interface() {
                                    if iface.interface_type == InterfaceType::Wireless {
                                        let name = iface.name.clone();
                                        let mode = NetworkManager::get_wireless_mode(&name)
                                            .unwrap_or(WirelessMode::Unknown);
                                        
                                        app.status_message = format!("Toggling monitor mode on {}...", name);
                                        
                                        let result = if mode == WirelessMode::Monitor {
                                            NetworkManager::disable_monitor_mode(&name).await
                                        } else {
                                            NetworkManager::enable_monitor_mode(&name).await
                                        };

                                        match result {
                                            Ok(_) => {
                                                app.status_message = format!(
                                                    "Monitor mode {} on {}",
                                                    if mode == WirelessMode::Monitor { "disabled" } else { "enabled" },
                                                    name
                                                );
                                                app.refresh_interfaces();
                                            }
                                            Err(e) => {
                                                app.status_message = format!("Error: {}", e);
                                            }
                                        }
                                    } else {
                                        app.status_message = "Not a wireless interface".to_string();
                                    }
                                }
                            }
                            KeyCode::Char('u') => {
                                // Bring interface up
                                if let Some(iface) = app.selected_interface() {
                                    let name = iface.name.clone();
                                    match NetworkManager::bring_up(&name).await {
                                        Ok(_) => {
                                            app.status_message = format!("{} is now UP", name);
                                            app.refresh_interfaces();
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Error: {}", e);
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('d') => {
                                // Bring interface down
                                if let Some(iface) = app.selected_interface() {
                                    let name = iface.name.clone();
                                    match NetworkManager::bring_down(&name).await {
                                        Ok(_) => {
                                            app.status_message = format!("{} is now DOWN", name);
                                            app.refresh_interfaces();
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Error: {}", e);
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('s') => {
                                // Scan for networks
                                if let Some(iface) = app.selected_interface() {
                                    if iface.interface_type == InterfaceType::Wireless {
                                        let name = iface.name.clone();
                                        app.status_message = format!("Scanning on {}...", name);
                                        
                                        let mut scanner = WifiScanner::new(&name);
                                        match scanner.scan().await {
                                            Ok(networks) => {
                                                app.networks = networks;
                                                if !app.networks.is_empty() {
                                                    app.network_state.select(Some(0));
                                                }
                                                app.status_message = format!(
                                                    "Found {} networks",
                                                    app.networks.len()
                                                );
                                                app.current_tab = 1; // Switch to networks tab
                                            }
                                            Err(e) => {
                                                app.status_message = format!("Scan error: {}", e);
                                            }
                                        }
                                    } else {
                                        app.status_message = "Select a wireless interface first".to_string();
                                    }
                                }
                            }
                            KeyCode::Char('n') => {
                                // Restart NetworkManager
                                app.status_message = "Restarting NetworkManager...".to_string();
                                match NetworkManager::restart_network_manager().await {
                                    Ok(_) => {
                                        app.status_message = "NetworkManager restarted".to_string();
                                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                        app.refresh_interfaces();
                                    }
                                    Err(e) => {
                                        app.status_message = format!("Error: {}", e);
                                    }
                                }
                            }
                            KeyCode::Char('M') => {
                                // Spoof MAC address
                                if let Some(iface) = app.selected_interface() {
                                    let name = iface.name.clone();
                                    let new_mac = NetworkManager::generate_random_mac();
                                    app.status_message = format!("Spoofing MAC on {} to {}...", name, new_mac);

                                    match NetworkManager::spoof_mac(&name, &new_mac).await {
                                        Ok(_) => {
                                            app.status_message = format!("MAC changed to {}", new_mac);
                                            app.refresh_interfaces();
                                        }
                                        Err(e) => {
                                            app.status_message = format!("Error: {}", e);
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('R') => {
                                // Enter rename mode
                                let iface_name = app.selected_interface().map(|i| i.name.clone());
                                if let Some(name) = iface_name {
                                    app.input_mode = InputMode::Rename;
                                    app.input_buffer = name.clone();
                                    app.status_message = format!("Enter new name for {} (Press Enter to confirm)", name);
                                }
                            }
                            _ => {}
                        },
                        InputMode::Rename | InputMode::MacInput | InputMode::ChannelInput => {
                            match key.code {
                                KeyCode::Enter => {
                                    // Process input
                                    if app.input_mode == InputMode::Rename {
                                        if let Some(iface) = app.selected_interface() {
                                            let old_name = iface.name.clone();
                                            let new_name = app.input_buffer.clone();

                                            if !new_name.is_empty() && new_name != old_name {
                                                match NetworkManager::rename_interface(&old_name, &new_name).await {
                                                    Ok(_) => {
                                                        app.status_message = format!("Renamed {} to {}", old_name, new_name);
                                                        app.refresh_interfaces();
                                                    }
                                                    Err(e) => {
                                                        app.status_message = format!("Error: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    app.input_mode = InputMode::Normal;
                                    app.input_buffer.clear();
                                }
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                    app.input_buffer.clear();
                                }
                                KeyCode::Char(c) => {
                                    app.input_buffer.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.input_buffer.pop();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Tabs
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  SOZIN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("v2.0.0", Style::default().fg(Color::DarkGray)),
            Span::raw(" │ "),
            Span::styled("Professional Network Interface Manager", Style::default().fg(Color::White)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(header, chunks[0]);

    // Tabs
    let tab_titles = vec!["Interfaces", "Networks", "Info"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[1]);

    // Main content based on tab
    match app.current_tab {
        0 => render_interfaces(f, app, chunks[2]),
        1 => render_networks(f, app, chunks[2]),
        2 => render_info(f, app, chunks[2]),
        _ => {}
    }

    // Status bar
    let status_style = if app.status_message.starts_with("Error") {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };
    
    let status = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status_message, status_style),
            Span::raw("  │  "),
            Span::styled("Press ? for help", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[3]);

    // Help popup
    if app.show_help {
        render_help_popup(f);
    }

    // Input mode popup
    if app.input_mode != InputMode::Normal {
        render_input_popup(f, app);
    }
}

fn render_interfaces(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Interface list
    let items: Vec<ListItem> = app
        .interfaces
        .iter()
        .map(|iface| {
            let state_color = match iface.state {
                InterfaceState::Up => Color::Green,
                InterfaceState::Down => Color::Red,
                InterfaceState::Unknown => Color::Yellow,
            };

            let type_icon = match iface.interface_type {
                InterfaceType::Wireless => "📶",
                InterfaceType::Ethernet => "🔌",
                InterfaceType::Loopback => "🔄",
                InterfaceType::Virtual => "🌐",
                InterfaceType::Unknown => "❓",
            };

            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", type_icon)),
                Span::styled(&iface.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(format!("[{}]", iface.state), Style::default().fg(state_color)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Interfaces "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[0], &mut app.interface_state.clone());

    // Interface details
    let details = if let Some(iface) = app.selected_interface() {
        let mode = if iface.interface_type == InterfaceType::Wireless {
            NetworkManager::get_wireless_mode(&iface.name)
                .map(|m| m.to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        } else {
            "N/A".to_string()
        };

        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&iface.name, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(iface.interface_type.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    iface.state.to_string(),
                    Style::default().fg(match iface.state {
                        InterfaceState::Up => Color::Green,
                        InterfaceState::Down => Color::Red,
                        InterfaceState::Unknown => Color::Yellow,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("MAC: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    iface.mac_address.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("IP: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    iface.ip_address.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Driver: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    iface.driver.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Mode: ", Style::default().fg(Color::DarkGray)),
                Span::styled(mode, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("─── Actions ───", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("m", Style::default().fg(Color::Cyan)),
                Span::raw(" Toggle Monitor  "),
                Span::styled("u", Style::default().fg(Color::Cyan)),
                Span::raw(" Up  "),
                Span::styled("d", Style::default().fg(Color::Cyan)),
                Span::raw(" Down"),
            ]),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Cyan)),
                Span::raw(" Scan WiFi  "),
                Span::styled("M", Style::default().fg(Color::Cyan)),
                Span::raw(" Spoof MAC"),
            ]),
        ]
    } else {
        vec![Line::from("No interface selected")]
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title(" Details "));
    f.render_widget(details_widget, chunks[1]);
}

fn render_networks(f: &mut Frame, app: &App, area: Rect) {
    if app.networks.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("No networks scanned yet."),
            Line::from(""),
            Line::from(vec![
                Span::raw("Select a wireless interface and press "),
                Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" to scan."),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title(" WiFi Networks "))
        .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("SSID").style(Style::default().fg(Color::Cyan)),
        Cell::from("BSSID").style(Style::default().fg(Color::Cyan)),
        Cell::from("CH").style(Style::default().fg(Color::Cyan)),
        Cell::from("Signal").style(Style::default().fg(Color::Cyan)),
        Cell::from("Security").style(Style::default().fg(Color::Cyan)),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .networks
        .iter()
        .map(|net| {
            let signal_color = if net.signal_strength > -50 {
                Color::Green
            } else if net.signal_strength > -70 {
                Color::Yellow
            } else {
                Color::Red
            };

            Row::new(vec![
                Cell::from(net.ssid.clone()),
                Cell::from(net.bssid.clone()),
                Cell::from(net.channel.to_string()),
                Cell::from(format!(
                    "{} {}dBm",
                    signal_to_bars(net.signal_strength),
                    net.signal_strength
                ))
                .style(Style::default().fg(signal_color)),
                Cell::from(net.security.to_string()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " WiFi Networks ({}) ",
        app.networks.len()
    )))
    .highlight_style(Style::default().bg(Color::DarkGray))
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.network_state.clone());
}

fn render_info(f: &mut Frame, _app: &App, area: Rect) {
    let info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  SOZIN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("- Professional Network Interface Manager", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Version: ", Style::default().fg(Color::DarkGray)),
            Span::styled("2.0.0", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Author: ", Style::default().fg(Color::DarkGray)),
            Span::styled("therealsylva", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  License: ", Style::default().fg(Color::DarkGray)),
            Span::styled("MIT", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Features:", Style::default().fg(Color::Cyan)),
        ]),
        Line::from("    • Monitor mode management"),
        Line::from("    • WiFi network scanning & discovery"),
        Line::from("    • Interface up/down control"),
        Line::from("    • MAC address spoofing"),
        Line::from("    • NetworkManager integration"),
        Line::from("    • Async architecture for performance"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ⚠ ", Style::default().fg(Color::Yellow)),
            Span::styled("Requires root privileges for network operations", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let info_widget = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title(" About "));
    f.render_widget(info_widget, area);
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("Keyboard Shortcuts", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("  Tab/Shift+Tab  Switch tabs"),
        Line::from("  j/↓            Move down"),
        Line::from("  k/↑            Move up"),
        Line::from("  q              Quit"),
        Line::from("  ?              Toggle help"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Interface Actions", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("  m              Toggle monitor mode"),
        Line::from("  u              Bring interface up"),
        Line::from("  d              Bring interface down"),
        Line::from("  R              Rename interface"),
        Line::from("  M              Spoof MAC address"),
        Line::from("  r              Refresh interfaces"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Scanning", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("  s              Scan for WiFi networks"),
        Line::from(""),
        Line::from(vec![
            Span::styled("System", Style::default().fg(Color::Yellow)),
        ]),
        Line::from("  n              Restart NetworkManager"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.input_mode {
        InputMode::Rename => "Rename Interface",
        _ => "Input",
    };

    let input_text = vec![
        Line::from(vec![
            Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("> "),
            Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Enter to confirm, Esc to cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let input = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(input, area);
}
