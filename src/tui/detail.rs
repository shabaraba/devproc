use crate::models::{ManagedProcess, ProcessMetrics};
use crate::process::ProcessManager;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_detail_view(
    f: &mut Frame,
    area: Rect,
    process: &ManagedProcess,
    manager: &ProcessManager,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Info
            Constraint::Length(5),  // Resources
            Constraint::Min(5),     // Command
        ])
        .split(area);

    render_info(f, chunks[0], process);
    render_resources(f, chunks[1], process, manager);
    render_command(f, chunks[2], process);
}

fn render_info(f: &mut Frame, area: Rect, process: &ManagedProcess) {
    let info_text = vec![
        Line::from(vec![
            Span::styled("PID:      ", Style::default().fg(Color::Yellow)),
            Span::raw(process.pid.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Name:     ", Style::default().fg(Color::Yellow)),
            Span::raw(&process.name),
        ]),
        Line::from(vec![
            Span::styled("Port:     ", Style::default().fg(Color::Yellow)),
            Span::raw(process.port.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string())),
        ]),
        Line::from(vec![
            Span::styled("CWD:      ", Style::default().fg(Color::Yellow)),
            Span::raw(process.cwd.to_string_lossy()),
        ]),
        Line::from(vec![
            Span::styled("Started:  ", Style::default().fg(Color::Yellow)),
            Span::raw(process.started_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" Process Info ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(info, area);
}

fn render_resources(
    f: &mut Frame,
    area: Rect,
    process: &ManagedProcess,
    manager: &ProcessManager,
) {
    let metrics = manager.get_metrics(process).unwrap_or_default();

    let cpu_color = if metrics.cpu_percent > 70.0 {
        Color::Red
    } else if metrics.cpu_percent > 30.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let mem_color = if metrics.memory_percent > 70.0 {
        Color::Red
    } else if metrics.memory_percent > 30.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let resources_text = vec![
        Line::from(vec![
            Span::styled("CPU:      ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.1}%", metrics.cpu_percent),
                Style::default().fg(cpu_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Memory:   ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} MB ({:.1}%)", metrics.memory_bytes / 1024 / 1024, metrics.memory_percent),
                Style::default().fg(mem_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status:   ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:?}", metrics.status)),
        ]),
    ];

    let resources = Paragraph::new(resources_text)
        .block(
            Block::default()
                .title(" Resources ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    f.render_widget(resources, area);
}

fn render_command(f: &mut Frame, area: Rect, process: &ManagedProcess) {
    let full_cmd = if process.args.is_empty() {
        process.command.clone()
    } else {
        format!("{} {}", process.command, process.args.join(" "))
    };

    let command_text = vec![
        Line::from(Span::raw(&full_cmd)),
    ];

    let command = Paragraph::new(command_text)
        .block(
            Block::default()
                .title(" Command ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(command, area);
}
