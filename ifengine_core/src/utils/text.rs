use std::sync::atomic::{AtomicPtr, Ordering};

pub type ProseFn = fn(&str) -> String;

/// Helper to convert a string slice or reference to &str without #[must_use] warnings.
#[inline(always)]
pub fn as_str(s: &str) -> &str {
    s
}

static PROSE_FN: AtomicPtr<()> =
    AtomicPtr::new(default_prose as fn(&str) -> String as *mut ());

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

pub fn split_braced(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut buf = String::new();
    let mut chars = s.chars().peekable();
    let mut inside = false;

    while let Some(c) = chars.next() {
        if !inside && c == '[' && chars.peek() == Some(&'[') {
            chars.next();
            result.push(buf.clone());
            buf.clear();
            inside = true;
        } else if inside && c == ']' && chars.peek() == Some(&']') {
            chars.next();
            if buf.is_empty() {
                buf = result.pop().unwrap_or_default();
            } else {
                result.push(buf.clone());
                buf.clear();
            }
            inside = false;
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        result.push(buf);
    }

    result
}

pub fn extract_braced_targets(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut chars = s.char_indices().peekable();
    let mut inside_start = None;

    while let Some((idx, c)) = chars.next() {
        if inside_start.is_none() && c == '[' && chars.peek().map(|(_, ch)| *ch) == Some('[') {
            chars.next();
            inside_start = Some(idx + 2);
        } else if let Some(start) = inside_start {
            if c == ']' && chars.peek().map(|(_, ch)| *ch) == Some(']') {
                let end = idx;
                chars.next();
                if start < end {
                    result.push(&s[start..end]);
                }
                inside_start = None;
            }
        }
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
            split_braced("Go to [[forest]] or [[inn]]"),
            vec!["Go to ", "forest", " or ", "inn"]
        );
        assert_eq!(
            split_braced("Go to [[forest]] or [[]]the [[inn]]"),
            vec!["Go to ", "forest", " or the ", "inn"]
        );
        assert_eq!(
            split_braced("[[]]hello [[world]]"),
            vec!["hello ", "world"]
        );
        assert_eq!(
            split_braced("hello[[]][[]]world"),
            vec!["helloworld"]
        );
        assert_eq!(
            split_braced("[[]]"),
            Vec::<String>::new()
        );
        assert_eq!(
            split_braced("hello [[]]"),
            vec!["hello "]
        );
    }

    #[test]
    fn test_extract_braced_targets() {
        assert_eq!(
            extract_braced_targets("Go to [[forest]] or [[inn]]"),
            vec!["forest", "inn"]
        );
        assert_eq!(
            extract_braced_targets("Go to [[forest]] or [[]]the [[inn]]"),
            vec!["forest", "inn"]
        );
        assert_eq!(
            extract_braced_targets("[[]]hello [[world]]"),
            vec!["world"]
        );
        assert_eq!(
            extract_braced_targets("[[]]"),
            Vec::<&str>::new()
        );
    }
}

