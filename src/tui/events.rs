use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use crate::scanner;
use crate::cache;
use crate::tmux;
use crate::commands::snapshot;

use super::app::{App, AppMode};

pub enum Action {
    Continue,
    Quit,
    Refresh,
}

/// Handle one key event. Returns what the main loop should do next.
pub fn handle_event(app: &mut App) -> Result<Action> {
    // Poll with a short timeout so we can auto-refresh
    if !event::poll(Duration::from_millis(250))? {
        return Ok(Action::Continue);
    }

    let ev = event::read()?;

    let Event::Key(key) = ev else {
        return Ok(Action::Continue);
    };

    // Ctrl-C always quits
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
        return Ok(Action::Quit);
    }

    match &app.mode {
        AppMode::Filter => handle_filter_key(app, key.code),
        AppMode::KillConfirm => return handle_kill_key(app, key.code),
        AppMode::Normal => return handle_normal_key(app, key.code),
    }

    Ok(Action::Continue)
}

fn handle_filter_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => app.clear_filter(),
        KeyCode::Enter => app.commit_filter(),
        KeyCode::Backspace => app.pop_filter_char(),
        KeyCode::Char(c) => app.push_filter_char(c),
        _ => {}
    }
}

fn handle_kill_key(app: &mut App, code: KeyCode) -> Result<Action> {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(pane) = app.selected_pane() {
                let pane_id = pane.pane_id.clone();
                let repo = pane.repo_display().to_string();

                // Pre-kill snapshot
                let snap_path = format!(
                    "/tmp/tmux-peek-kill-{}-{}.md",
                    repo,
                    chrono::Utc::now().format("%Y%m%d-%H%M%S")
                );
                snapshot::run(false, Some(snap_path.clone())).ok();

                tmux::kill_pane(&pane_id)?;
                app.cancel_kill();
                app.set_flash(format!("Killed {}. Snapshot: {}", pane_id, snap_path));
                return Ok(Action::Refresh);
            }
            app.cancel_kill();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_kill();
        }
        _ => {}
    }
    Ok(Action::Continue)
}

fn handle_normal_key(app: &mut App, code: KeyCode) -> Result<Action> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::Quit),

        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),

        KeyCode::Enter => {
            if let Some(pane) = app.selected_pane() {
                let session = pane.session_name.clone();
                let window_id = pane.window_id.clone();
                let pane_id = pane.pane_id.clone();
                tmux::jump_to_pane(&session, &window_id, &pane_id).ok();
            }
        }

        KeyCode::Char('p') => app.toggle_preview(),

        KeyCode::Char('s') => {
            // Write snapshot to /tmp and flash path
            let path = format!(
                "/tmp/tmux-peek-{}.md",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            );
            match snapshot::run(false, Some(path.clone())) {
                Ok(_) => app.set_flash(format!("Snapshot → {}", path)),
                Err(e) => app.set_flash(format!("Snapshot failed: {}", e)),
            }
        }

        KeyCode::Char('k') => app.start_kill_confirm(),

        KeyCode::Char('/') => app.start_filter(),

        KeyCode::Char('1') => app.toggle_attention_only(),
        KeyCode::Char('2') => {
            app.attention_only = false;
            app.rebuild_visible();
        }

        KeyCode::Char('r') | KeyCode::F(5) => return Ok(Action::Refresh),

        _ => {}
    }
    Ok(Action::Continue)
}

/// Perform a full re-scan and update the app state.
pub fn do_refresh(app: &mut App) {
    match scanner::scan() {
        Ok(new_state) => {
            cache::save(&new_state).ok();
            app.clear_flash();
            app.refresh(new_state);
        }
        Err(e) => {
            app.set_flash(format!("Refresh failed: {}", e));
        }
    }
}
