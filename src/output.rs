use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

static NO_COLOR: AtomicBool = AtomicBool::new(false);

pub fn init_color(force_no_color: bool) {
    if force_no_color
        || std::env::var_os("NO_COLOR").is_some()
        || !std::io::stdout().is_terminal()
    {
        NO_COLOR.store(true, Ordering::Relaxed);
    }
}

pub fn is_no_color() -> bool {
    NO_COLOR.load(Ordering::Relaxed)
}

pub fn bold(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[1m{s}\x1b[0m")
    }
}

pub fn dim(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[2m{s}\x1b[0m")
    }
}

pub fn cyan(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[36m{s}\x1b[0m")
    }
}

pub fn green(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[32m{s}\x1b[0m")
    }
}

pub fn yellow(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[33m{s}\x1b[0m")
    }
}

#[allow(dead_code)]
pub fn red(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[31m{s}\x1b[0m")
    }
}

pub fn magenta(s: &str) -> String {
    if is_no_color() {
        s.to_string()
    } else {
        format!("\x1b[35m{s}\x1b[0m")
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Heuristic: ~4 characters per token for standard English prose/markdown,
    // clamped to word count * 1.3 for code/symbols.
    let by_chars = text.len() / 4;
    let words = text.split_whitespace().count();
    let by_words = (words as f64 * 1.35) as usize;
    std::cmp::max(std::cmp::max(by_chars, by_words), 1)
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn format_tokens(tokens: usize) -> String {
    if tokens < 1000 {
        format!("~{tokens} tok")
    } else {
        format!("~{:.1}k tok", tokens as f64 / 1000.0)
    }
}

pub fn terminal_width() -> usize {
    if let Ok(col_str) = std::env::var("COLUMNS") {
        if let Ok(cols) = col_str.parse::<usize>() {
            if cols >= 40 {
                return cols;
            }
        }
    }
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col as usize;
        }
    }
    100
}

pub fn safe_truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    let clean = s.lines().next().unwrap_or(s).trim();
    if clean.chars().count() > max_chars {
        let prefix = safe_truncate_chars(clean, max_chars.saturating_sub(3));
        format!("{prefix}...")
    } else {
        clean.to_string()
    }
}

pub fn wrap_text(text: &str, width: usize, indent: &str) -> String {
    let mut result = String::new();
    let indent_chars = indent.chars().count();
    let max_line = width.saturating_sub(indent_chars).max(20);

    for paragraph in text.split('\n') {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }

        let mut current_line = String::new();
        let words: Vec<&str> = trimmed.split_whitespace().collect();

        for word in words {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.chars().count() + 1 + word.chars().count() <= max_line {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                if !result.is_empty() {
                    result.push('\n');
                    result.push_str(indent);
                }
                result.push_str(&current_line);
                current_line.clear();
                current_line.push_str(word);
            }
        }

        if !current_line.is_empty() {
            if !result.is_empty() {
                result.push('\n');
                result.push_str(indent);
            }
            result.push_str(&current_line);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        let s = "Hello world this is a test string";
        assert!(estimate_tokens(s) >= 7);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "~500 tok");
        assert_eq!(format_tokens(2500), "~2.5k tok");
    }

    #[test]
    fn test_truncate_ellipsis_multibyte_safe() {
        // String with em-dash at byte boundary 40..43
        let s = "All animation knowledge for HyperFrames — atomic motion rules";
        let res = truncate_ellipsis(s, 42);
        assert!(res.ends_with("..."));
        assert!(!res.contains('\u{FFFD}'));

        // Emoji test
        let emoji_s = "🚀✨🔥 Testing emojis and symbols across lines";
        let emoji_res = truncate_ellipsis(emoji_s, 5);
        assert!(emoji_res.ends_with("..."));
    }

    #[test]
    fn test_wrap_text() {
        let text = "One two three four five six seven eight nine ten";
        let wrapped = wrap_text(text, 20, "  ");
        for line in wrapped.lines() {
            assert!(line.chars().count() <= 22);
        }
    }
}
