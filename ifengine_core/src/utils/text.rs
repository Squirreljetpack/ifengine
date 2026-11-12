use std::sync::atomic::{AtomicPtr, Ordering};

pub type ProseFn = fn(&str) -> String;

/// Helper to convert a string slice or reference to &str without #[must_use] warnings.
#[inline(always)]
pub fn as_str(s: &str) -> &str {
    s
}

static PROSE_FN: AtomicPtr<()> = AtomicPtr::new(default_prose as fn(&str) -> String as *mut ());

/// Sets the global prose transformation function pointer.
///
/// By default, this points to [`default_prose`].
pub fn set_prose_fn(f: ProseFn) {
    PROSE_FN.store(f as *mut (), Ordering::Relaxed);
}

/// Resets the global prose transformation function pointer to [`default_prose`].
pub fn reset_prose_fn() {
    set_prose_fn(default_prose);
}

/// Transforms input text using the active global prose function pointer.
pub fn prose(text: &str) -> String {
    let ptr = PROSE_FN.load(Ordering::Relaxed);
    let f: ProseFn = unsafe { std::mem::transmute(ptr) };
    f(text)
}

/// Default prose processor:
/// - Replaces '' with "
/// - Converts straight quotes to curly quotes
/// - Converts -- to em-dash (—)
/// - Converts ... to ellipsis (…)
pub fn default_prose(text: &str) -> String {
    let mut result = text.to_string();

    result = result.replace("''", "\"");

    result = result.replace("--", "—");

    result = result.replace("...", "…");

    // Double quotes
    let mut final_text = String::with_capacity(result.len());
    let mut in_double = false;
    let mut escape = false;

    for c in result.chars() {
        if escape {
            match c {
                '"' | '\'' | '\\' => final_text.push(c),
                _ => {
                    final_text.push('\\');
                    final_text.push(c);
                }
            }
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            '"' => {
                if in_double {
                    final_text.push('”');
                } else {
                    final_text.push('“');
                }
                in_double = !in_double;
            }
            '\'' => final_text.push('’'),
            _ => final_text.push(c),
        }
    }

    if escape {
        final_text.push('\\');
    }

    final_text
}

/// trim start and end of each line, and empty lines around
pub fn trim_lines(s: &str) -> String {
    let lines: Vec<&str> = s
        .lines()
        .map(str::trim) // trim each line
        .collect();

    let start = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
    let end = lines.iter().rposition(|line| !line.is_empty()).unwrap_or(0);

    lines[start..=end].join("\n")
}

/// Splits `s` into alternating plain/link parts: even indices are plain text, odd indices
/// are `[link]` targets. `[[` and `]]` are literal bracket escapes.
///
/// An unclosed `[` treats the trailing text as a target; an unclosed `]` before any `[`
/// treats the leading text as a target.
pub fn split_braced(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut buf = String::new();
    let mut chars = s.chars().peekable();
    let mut inside = false;

    // Check if there is an unclosed `[` with no matching `]` after it,
    // or an unclosed `]` at the start with no preceding `[`.
    let starts_with_unclosed_close = if let Some(close_idx) = s.find(']') {
        match s.find('[') {
            Some(open_idx) => close_idx < open_idx,
            None => true,
        }
    } else {
        false
    };

    if starts_with_unclosed_close {
        // Treat the opening portion up to `]` as braced. The empty leading part keeps the
        // even/odd outside/inside parity contract expected by consumers.
        result.push(String::new());
        inside = true;
    }

    while let Some(c) = chars.next() {
        if c == '[' {
            if chars.peek() == Some(&'[') {
                chars.next();
                buf.push('[');
            } else if !inside {
                result.push(buf.clone());
                buf.clear();
                inside = true;
            } else {
                buf.push(c);
            }
        } else if c == ']' {
            if chars.peek() == Some(&']') {
                chars.next();
                buf.push(']');
            } else if inside {
                result.push(buf.clone());
                buf.clear();
                inside = false;
            } else {
                buf.push(c);
            }
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        result.push(buf);
    }

    result
}

pub fn find_hash_match<I, S>(strings: I, target: u64) -> Option<S>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    strings
        .into_iter()
        .find(|s| const_fnv1a_hash::fnv1a_hash_str_64(s.as_ref()) == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prose_typography() {
        assert_eq!(default_prose("Hello... world--test"), "Hello… world—test");
        assert_eq!(default_prose("\"Double quotes\""), "“Double quotes”");
        assert_eq!(default_prose("It's fine"), "It’s fine");
    }

    #[test]
    fn test_default_prose_double_single_quotes() {
        assert_eq!(default_prose("She said, ''Hello!''"), "She said, “Hello!”");
        assert_eq!(
            default_prose("''Don't go,'' she warned."),
            "“Don’t go,” she warned."
        );
    }

    #[test]
    fn test_set_and_reset_prose_fn() {
        assert_eq!(prose("test--value"), "test—value");

        fn custom_hook(text: &str) -> String {
            if text == "__custom_hook_probe__" {
                "custom_hook_result".to_string()
            } else {
                default_prose(text)
            }
        }

        set_prose_fn(custom_hook);
        assert_eq!(prose("__custom_hook_probe__"), "custom_hook_result");
        assert_eq!(prose("test--value"), "test—value");

        reset_prose_fn();
        assert_eq!(prose("__custom_hook_probe__"), "__custom_hook_probe__");
        assert_eq!(prose("test--value"), "test—value");
    }

    #[test]
    fn test_split_braced_and_skip_empty() {
        assert_eq!(
            split_braced("Go to [forest] or [inn]"),
            vec!["Go to ", "forest", " or ", "inn"]
        );
        assert_eq!(
            split_braced("Literal [[bracket]] test [link]"),
            vec!["Literal [bracket] test ", "link"]
        );
        assert_eq!(
            split_braced("unclosed [left end"),
            vec!["unclosed ", "left end"]
        );
        // A `]` before any `[` opens a braced section at the start of the string; the empty
        // leading part preserves the even/odd outside/inside parity.
        let parts = split_braced("a] b [c]");
        assert_eq!(parts, vec!["", "a", " b ", "c"]);
        assert_eq!(parts[1], "a");
        assert_eq!(parts[3], "c");
        assert_eq!(
            split_braced("unclosed right end] after"),
            vec!["", "unclosed right end", " after"]
        );
    }
}
