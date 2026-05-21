use crate::types::{AgentPane, AgentStatus, State};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filter,
    KillConfirm,
}

pub struct App {
    pub state: State,
    /// Flat list of visible panes (after filter)
    pub visible: Vec<usize>,
    /// Selected index into `visible`
    pub selected: usize,
    /// Current filter string
    pub filter: String,
    pub mode: AppMode,
    /// Whether the preview pane is shown
    pub show_preview: bool,
    /// Only show agents that need attention
    pub attention_only: bool,
    /// Message to show in status bar (cleared on next tick)
    pub flash: Option<String>,
}

impl App {
    pub fn new(state: State) -> Self {
        let visible = (0..state.panes.len()).collect::<Vec<_>>();
        let selected = 0;
        App {
            state,
            visible,
            selected,
            filter: String::new(),
            mode: AppMode::Normal,
            show_preview: true,
            attention_only: false,
            flash: None,
        }
    }

    pub fn refresh(&mut self, new_state: State) {
        // Preserve selection by pane_id if possible
        let selected_id = self.selected_pane().map(|p| p.pane_id.clone());
        self.state = new_state;
        self.rebuild_visible();

        if let Some(id) = selected_id {
            if let Some(pos) = self.visible.iter().position(|&i| self.state.panes[i].pane_id == id) {
                self.selected = pos;
                return;
            }
        }
        // Clamp selection
        if !self.visible.is_empty() && self.selected >= self.visible.len() {
            self.selected = self.visible.len() - 1;
        }
    }

    pub fn rebuild_visible(&mut self) {
        let filter = self.filter.to_lowercase();
        self.visible = self
            .state
            .panes
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if self.attention_only && p.status != AgentStatus::NeedsAttention {
                    return false;
                }
                if filter.is_empty() {
                    return true;
                }
                let haystack = format!(
                    "{} {} {} {}",
                    p.display_name(),
                    p.repo_display(),
                    p.branch_display(),
                    p.status_reason
                )
                .to_lowercase();
                haystack.contains(&filter)
            })
            .map(|(i, _)| i)
            .collect();

        if self.selected >= self.visible.len() && !self.visible.is_empty() {
            self.selected = self.visible.len() - 1;
        }
    }

    pub fn selected_pane(&self) -> Option<&AgentPane> {
        self.visible
            .get(self.selected)
            .and_then(|&i| self.state.panes.get(i))
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.visible.is_empty() && self.selected < self.visible.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    pub fn toggle_attention_only(&mut self) {
        self.attention_only = !self.attention_only;
        self.rebuild_visible();
        self.selected = 0;
    }

    pub fn start_filter(&mut self) {
        self.mode = AppMode::Filter;
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.rebuild_visible();
        self.selected = 0;
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.rebuild_visible();
        self.selected = 0;
    }

    pub fn commit_filter(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.mode = AppMode::Normal;
        self.rebuild_visible();
        self.selected = 0;
    }

    pub fn start_kill_confirm(&mut self) {
        if self.selected_pane().is_some() {
            self.mode = AppMode::KillConfirm;
        }
    }

    pub fn cancel_kill(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn set_flash(&mut self, msg: impl Into<String>) {
        self.flash = Some(msg.into());
    }

    pub fn clear_flash(&mut self) {
        self.flash = None;
    }
}
