/// Parse ANSI SGR escape sequences and convert to ratatui styled `Line`s.
use std::sync::OnceLock;

use regex::Regex;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Convert a multi-line string containing ANSI escape codes into ratatui `Line`s.
pub fn parse_ansi(text: &str) -> Vec<Line<'static>> {
    text.lines().map(parse_ansi_line).collect()
}

fn csi_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Matches any CSI sequence (ESC [ ... letter) or OSC/other two-char escapes
    RE.get_or_init(|| {
        Regex::new(r"\x1b\[[0-9;?<>!]*[A-Za-z]|\x1b[^\[\r\n\x1b]|\x1b$").unwrap()
    })
}

fn parse_ansi_line(line: &str) -> Line<'static> {
    let re = csi_regex();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut style = Style::default();
    let mut last_end = 0;

    for m in re.find_iter(line) {
        let text_before = &line[last_end..m.start()];
        if !text_before.is_empty() {
            spans.push(Span::styled(text_before.to_owned(), style));
        }

        let seq = m.as_str();
        // Only SGR sequences end in 'm' — everything else is a cursor/erase command
        if seq.ends_with('m') && seq.len() > 2 {
            let inner = &seq[2..seq.len() - 1]; // strip ESC[ and trailing m
            style = apply_sgr(style, inner);
        }

        last_end = m.end();
    }

    let remaining = &line[last_end..];
    if !remaining.is_empty() {
        spans.push(Span::styled(remaining.to_owned(), style));
    }

    if spans.is_empty() {
        Line::from(line.to_owned())
    } else {
        Line::from(spans)
    }
}

fn apply_sgr(mut style: Style, codes_str: &str) -> Style {
    if codes_str.is_empty() {
        return Style::reset();
    }

    let codes: Vec<u32> = codes_str
        .split(';')
        .filter_map(|s| s.parse().ok())
        .collect();

    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => style = Style::reset(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            5 | 6 => style = style.add_modifier(Modifier::SLOW_BLINK),
            7 => style = style.add_modifier(Modifier::REVERSED),
            9 => style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            25 => style = style.remove_modifier(Modifier::SLOW_BLINK),
            27 => style = style.remove_modifier(Modifier::REVERSED),

            // Standard foreground (30–37)
            30 => style = style.fg(Color::Black),
            31 => style = style.fg(Color::Red),
            32 => style = style.fg(Color::Green),
            33 => style = style.fg(Color::Yellow),
            34 => style = style.fg(Color::Blue),
            35 => style = style.fg(Color::Magenta),
            36 => style = style.fg(Color::Cyan),
            37 => style = style.fg(Color::White),
            38 if codes.get(i + 1) == Some(&5) && codes.get(i + 2).is_some() => {
                style = style.fg(Color::Indexed(codes[i + 2] as u8));
                i += 2;
            }
            38 if codes.get(i + 1) == Some(&2)
                && codes.get(i + 2).is_some()
                && codes.get(i + 3).is_some()
                && codes.get(i + 4).is_some() =>
            {
                style = style.fg(Color::Rgb(
                    codes[i + 2] as u8,
                    codes[i + 3] as u8,
                    codes[i + 4] as u8,
                ));
                i += 4;
            }
            39 => style = style.fg(Color::Reset),

            // Standard background (40–47)
            40 => style = style.bg(Color::Black),
            41 => style = style.bg(Color::Red),
            42 => style = style.bg(Color::Green),
            43 => style = style.bg(Color::Yellow),
            44 => style = style.bg(Color::Blue),
            45 => style = style.bg(Color::Magenta),
            46 => style = style.bg(Color::Cyan),
            47 => style = style.bg(Color::White),
            48 if codes.get(i + 1) == Some(&5) && codes.get(i + 2).is_some() => {
                style = style.bg(Color::Indexed(codes[i + 2] as u8));
                i += 2;
            }
            48 if codes.get(i + 1) == Some(&2)
                && codes.get(i + 2).is_some()
                && codes.get(i + 3).is_some()
                && codes.get(i + 4).is_some() =>
            {
                style = style.bg(Color::Rgb(
                    codes[i + 2] as u8,
                    codes[i + 3] as u8,
                    codes[i + 4] as u8,
                ));
                i += 4;
            }
            49 => style = style.bg(Color::Reset),

            // Bright foreground (90–97)
            90 => style = style.fg(Color::DarkGray),
            91 => style = style.fg(Color::LightRed),
            92 => style = style.fg(Color::LightGreen),
            93 => style = style.fg(Color::LightYellow),
            94 => style = style.fg(Color::LightBlue),
            95 => style = style.fg(Color::LightMagenta),
            96 => style = style.fg(Color::LightCyan),
            97 => style = style.fg(Color::White),

            // Bright background (100–107)
            100 => style = style.bg(Color::DarkGray),
            101 => style = style.bg(Color::LightRed),
            102 => style = style.bg(Color::LightGreen),
            103 => style = style.bg(Color::LightYellow),
            104 => style = style.bg(Color::LightBlue),
            105 => style = style.bg(Color::LightMagenta),
            106 => style = style.bg(Color::LightCyan),
            107 => style = style.bg(Color::White),

            _ => {}
        }
        i += 1;
    }
    style
}
