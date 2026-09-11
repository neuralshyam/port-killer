use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title & Search bar
            Constraint::Min(8),    // Port Table & Process Details
            Constraint::Length(1), // Status Bar / Errors
            Constraint::Length(1), // Help / Shortcut keys
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_status(f, app, chunks[2]);
    render_help(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let search_style = if app.is_searching {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let search_text = if app.search_query.is_empty() && !app.is_searching {
        " Press '/' to filter ports, processes, frameworks, or directories...".to_string()
    } else {
        format!(" {}", app.search_query)
    };

    let title = format!(
        " ⚡ KPORT v{} [Active Ports: {}] ",
        env!("CARGO_PKG_VERSION"),
        app.filtered_ports.len()
    );

    let search_bar = Paragraph::new(search_text)
        .style(search_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
                .border_style(search_style),
        );

    f.render_widget(search_bar, area);
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Render Table of Ports
    let rows: Vec<Row> = app
        .filtered_ports
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.selected_index;
            let is_marked = app.selected_to_kill.contains(&p.pid);

            let mark_symbol = if is_marked { "[x] " } else { "    " };

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(30, 50, 90))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_marked {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let port_styled = Span::styled(
                format!("{}{}", mark_symbol, p.port),
                if p.port < 1024 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Cyan)
                },
            );

            let proto_styled = Span::styled(p.protocol.to_string(), Style::default().fg(Color::DarkGray));
            let pid_styled = Span::styled(p.pid.to_string(), Style::default().fg(Color::Magenta));
            let name_styled = Span::styled(&p.name, Style::default().fg(Color::White));

            let project_badge = if let Some(ref fw) = p.framework {
                Span::styled(format!("[{}]", fw), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else if let Some(ref proj) = p.project_name {
                Span::styled(format!("[{}]", proj), Style::default().fg(Color::LightCyan))
            } else {
                Span::styled(p.memory_human(), Style::default().fg(Color::LightBlue))
            };

            Row::new(vec![
                Line::from(port_styled),
                Line::from(proto_styled),
                Line::from(pid_styled),
                Line::from(name_styled),
                Line::from(project_badge),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec!["  PORT", "PROTO", "PID", "PROCESS", "FRAMEWORK / STACK"])
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Min(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Active Listening Ports ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(table, body_chunks[0]);

    // Render Process Details Panel
    if let Some(selected) = app.filtered_ports.get(app.selected_index) {
        let mut details = vec![
            Line::from(vec![
                Span::styled("Port: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(":{}", selected.port), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" ("),
                Span::styled(selected.protocol.to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(")"),
            ]),
            Line::from(vec![
                Span::styled("Process: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&selected.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(format!("PID: {}", selected.pid), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Framework: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    selected.framework.as_deref().unwrap_or("Generic / Unknown"),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().fg(Color::DarkGray)),
                Span::styled(selected.memory_human(), Style::default().fg(Color::LightBlue)),
                Span::raw("  "),
                Span::styled("User: ", Style::default().fg(Color::DarkGray)),
                Span::styled(selected.user.as_deref().unwrap_or("unknown"), Style::default().fg(Color::LightGreen)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                if selected.is_system_protected {
                    Span::styled("🔒 Protected System Daemon", Style::default().fg(Color::Red))
                } else if selected.is_orphan {
                    Span::styled("🧟 Orphan Process (Parent Dead)", Style::default().fg(Color::LightRed))
                } else {
                    Span::styled("🟢 Active Dev Service", Style::default().fg(Color::Green))
                },
            ]),
            Line::from(vec![
                Span::styled("Directory: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    selected.cwd.as_deref().unwrap_or("Unavailable"),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        // Probe result if triggered
        if let Some(ref pr) = app.probe_result {
            details.push(Line::from(""));
            details.push(Line::from(Span::styled(
                "⚡ Live Probe Response:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )));
            if let Some(ref status) = pr.http_status {
                details.push(Line::from(vec![
                    Span::styled("  HTTP: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(status, Style::default().fg(Color::LightGreen)),
                ]));
            }
            if let Some(ref srv) = pr.server_header {
                details.push(Line::from(vec![
                    Span::styled("  Server: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(srv, Style::default().fg(Color::White)),
                ]));
            }
            if let Some(ref title) = pr.html_title {
                details.push(Line::from(vec![
                    Span::styled("  Title: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("\"{}\"", title), Style::default().fg(Color::LightCyan)),
                ]));
            }
        }

        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "Command Line:",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
        )));
        details.push(Line::from(Span::styled(
            if selected.cmdline.is_empty() { &selected.name } else { &selected.cmdline },
            Style::default().fg(Color::Gray),
        )));

        let details_panel = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Process & Project Inspector ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(details_panel, body_chunks[1]);
    } else {
        let empty = Paragraph::new("No matching ports found.")
            .block(Block::default().borders(Borders::ALL).title(" Process & Project Inspector "));
        f.render_widget(empty, body_chunks[1]);
    }
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    if let Some((ref msg, is_error)) = app.status_message {
        let style = if is_error {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        };
        let p = Paragraph::new(format!("  {}", msg)).style(style);
        f.render_widget(p, area);
    }
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.is_searching {
        " [Enter] Apply Search | [Esc] Cancel Search "
    } else {
        " [↑/↓ or j/k] Navigate | [/] Search | [p] Probe Port | [Space] Select | [Enter] Kill | [K] Force Kill | [r] Refresh | [q] Quit "
    };

    let p = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}
