use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::types::AgentStatus;
use super::app::{App, AppMode};

const COL_INDICATOR: u16 = 2;
const COL_AGENT: u16 = 9;
const COL_REPO: u16 = 14;
const COL_BRANCH: u16 = 18;
const COL_TIME: u16 = 6;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = if app.show_preview {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Length(10),
                Constraint::Length(1),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Length(1),
            ])
            .split(area)
    };

    draw_main(frame, app, chunks[0]);

    if app.show_preview {
        draw_preview(frame, app, chunks[1]);
        draw_statusbar(frame, app, chunks[2]);
    } else {
        draw_statusbar(frame, app, chunks[1]);
    }

    if app.mode == AppMode::KillConfirm {
        draw_kill_confirm(frame, app, area);
    }
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    let total = app.state.panes.len();
    let attention = app.state.attention_count();

    let title = if attention > 0 {
        format!(
            " tmux.peek ─ {} agents · {} need you ",
            total, attention
        )
    } else {
        format!(" tmux.peek ─ {} agents ", total)
    };

    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.visible.is_empty() {
        let msg = if app.filter.is_empty() {
            Paragraph::new("No agent panes found. Are you inside a tmux session?")
                .style(Style::default().fg(Color::DarkGray))
        } else {
            Paragraph::new(format!("No results for '{}'", app.filter))
                .style(Style::default().fg(Color::DarkGray))
        };
        frame.render_widget(msg, inner);
        return;
    }

    // Build rows with group headers
    let mut rows: Vec<Row> = Vec::new();
    let mut row_to_visible: Vec<Option<usize>> = Vec::new(); // row index -> visible index
    let mut current_group: Option<AgentStatus> = None;

    for (vis_idx, &pane_idx) in app.visible.iter().enumerate() {
        let pane = &app.state.panes[pane_idx];

        // Insert group header when status changes
        if current_group.as_ref() != Some(&pane.status) {
            current_group = Some(pane.status.clone());

            let group_count = app
                .visible
                .iter()
                .filter(|&&i| app.state.panes[i].status == pane.status)
                .count();

            let header_style = group_header_style(&pane.status);
            let header_text = format!(" {} [{}]", pane.status.label(), group_count);

            rows.push(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(Span::styled(header_text, header_style)).into(),
                    Cell::from("").into(),
                    Cell::from("").into(),
                    Cell::from("").into(),
                    Cell::from("").into(),
                ])
                .height(1),
            );
            row_to_visible.push(None);

            // Separator
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(Span::styled(
                    "─".repeat(60),
                    Style::default().fg(Color::DarkGray),
                )).into(),
                Cell::from("").into(),
                Cell::from("").into(),
                Cell::from("").into(),
                Cell::from("").into(),
            ]));
            row_to_visible.push(None);
        }

        let is_selected = vis_idx == app.selected;
        let base_style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let indicator = if is_selected { "▶" } else { " " };
        let agent_style = agent_color(&pane.agent.as_deref().unwrap_or(""));
        let reason_style = reason_style(&pane.status);

        let row = Row::new(vec![
            Cell::from(indicator).style(Style::default().fg(Color::Yellow)),
            Cell::from(Span::styled(pane.display_name().to_string(), agent_style)),
            Cell::from(pane.repo_display().to_string()),
            Cell::from(pane.branch_display().to_string()),
            Cell::from(Span::styled(
                pane.elapsed_display(),
                Style::default().fg(Color::DarkGray),
            )),
            Cell::from(Span::styled(pane.reason_display(), reason_style)),
        ])
        .style(base_style)
        .height(1);

        rows.push(row);
        row_to_visible.push(Some(vis_idx));
    }

    // Find the table row that corresponds to the selected visible item
    let selected_row = row_to_visible
        .iter()
        .position(|v| *v == Some(app.selected));

    let widths = [
        Constraint::Length(COL_INDICATOR),
        Constraint::Length(COL_AGENT),
        Constraint::Length(COL_REPO),
        Constraint::Length(COL_BRANCH),
        Constraint::Length(COL_TIME),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .column_spacing(1);

    let mut table_state = TableState::default();
    table_state.select(selected_row);

    frame.render_stateful_widget(table, inner, &mut table_state);
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content) = match app.selected_pane() {
        Some(pane) => {
            let t = format!(
                " preview · {} · {} ",
                pane.repo_display(),
                pane.pane_id
            );
            let lines: Vec<Line> = pane
                .last_output_lines
                .iter()
                .rev()
                .take(area.height.saturating_sub(2) as usize)
                .rev()
                .map(|l| Line::from(Span::raw(l.clone())))
                .collect();
            (t, lines)
        }
        None => (" preview ".to_string(), vec![]),
    };

    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(paragraph, area);
}

fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let text = match &app.mode {
        AppMode::Filter => {
            let filter_display = format!("/{}", app.filter);
            Line::from(vec![
                Span::styled("filter: ", Style::default().fg(Color::Yellow)),
                Span::raw(filter_display),
                Span::styled("█", Style::default().fg(Color::Yellow)),
                Span::raw("  esc:cancel  enter:apply"),
            ])
        }
        AppMode::KillConfirm => Line::from(Span::styled(
            "Kill this agent? Press y to confirm, n to cancel",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        _ => {
            if let Some(flash) = &app.flash {
                Line::from(Span::styled(
                    flash.as_str(),
                    Style::default().fg(Color::Green),
                ))
            } else {
                let attn_label = if app.attention_only { "2:all" } else { "1:attn" };
                Line::from(vec![
                    keybind("enter", "jump"),
                    Span::raw("  "),
                    keybind("p", "preview"),
                    Span::raw("  "),
                    keybind("s", "snapshot"),
                    Span::raw("  "),
                    keybind("k", "kill"),
                    Span::raw("  "),
                    keybind("/", "filter"),
                    Span::raw("  "),
                    keybind("1", attn_label),
                    Span::raw("  "),
                    keybind("r", "refresh"),
                    Span::raw("  "),
                    keybind("q", "quit"),
                ])
            }
        }
    };

    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(para, area);
}

fn draw_kill_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(50, 7, area);
    frame.render_widget(Clear, popup);

    let pane_info = app
        .selected_pane()
        .map(|p| format!("{} @ {} ({})", p.display_name(), p.repo_display(), p.pane_id))
        .unwrap_or_default();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Kill agent pane?",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  {}", pane_info)),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("y", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(":kill   "),
            Span::styled("n", Style::default().fg(Color::Green)),
            Span::raw("/esc:cancel"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Confirm ");

    let para = Paragraph::new(text).block(block);
    frame.render_widget(para, popup);
}

fn keybind<'a>(key: &'a str, action: &'a str) -> Span<'a> {
    Span::raw(format!("{}:{}", key, action))
}

fn group_header_style(status: &AgentStatus) -> Style {
    match status {
        AgentStatus::NeedsAttention => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        AgentStatus::Error => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        AgentStatus::Running => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        AgentStatus::Done => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
        AgentStatus::Unknown => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    }
}

fn agent_color(agent: &str) -> Style {
    match agent {
        "claude" => Style::default().fg(Color::Cyan),
        "codex" => Style::default().fg(Color::Blue),
        "aider" => Style::default().fg(Color::Magenta),
        "hermes" => Style::default().fg(Color::Yellow),
        "gemini" => Style::default().fg(Color::Blue),
        "goose" => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::White),
    }
}

fn reason_style(status: &AgentStatus) -> Style {
    match status {
        AgentStatus::NeedsAttention => Style::default().fg(Color::Yellow),
        AgentStatus::Error => Style::default().fg(Color::Red),
        AgentStatus::Running => Style::default().fg(Color::Green),
        AgentStatus::Done => Style::default().fg(Color::DarkGray),
        AgentStatus::Unknown => Style::default().fg(Color::DarkGray),
    }
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let x = area.x + (area.width - popup_width) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, popup_width, height.min(area.height))
}
