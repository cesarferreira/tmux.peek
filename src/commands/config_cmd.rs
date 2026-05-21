use anyhow::Result;

use crate::config::Config;

pub fn run(init: bool, show: bool) -> Result<()> {
    if init {
        let path = Config::write_default()?;
        println!("Config written to: {}", path.display());
        println!("Edit it to customise agent names, session filters, and patterns.");
        return Ok(());
    }

    if show {
        let path = Config::path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            print!("{}", content);
        } else {
            println!("No config file found at {}", path.display());
            println!("Run `tmux.peek config --init` to create one.");
        }
        return Ok(());
    }

    // Default: show path and status
    let path = Config::path();
    println!("Config path: {}", path.display());
    if path.exists() {
        let cfg = Config::load();
        println!("  extra_agents:      {:?}", cfg.extra_agents);
        println!("  exclude_sessions:  {:?}", cfg.exclude_sessions);
        println!("  include_sessions:  {:?}", cfg.include_sessions);
        println!("  attention_patterns: {} custom", cfg.attention_patterns.len());
        println!("  done_patterns:      {} custom", cfg.done_patterns.len());
        println!("  status_format:     {}", cfg.status_format);
    } else {
        println!("  (no config file — using defaults)");
    }

    Ok(())
}
