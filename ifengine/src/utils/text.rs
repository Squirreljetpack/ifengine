/// - Replaces {digit} with words
/// - Converts straight quotes to curly quotes
/// - Converts -- to em-dash (—)
/// - Converts ... to ellipsis (…)
pub fn linguate(text: &str) -> String {
    // let re_digits = regex::Regex::new(r"\{(\d+)\}").unwrap();
    // let text = re_digits.replace_all(text, |caps: &regex::Captures<'_>| {
    //     let n: u64 = caps[1].parse().unwrap();
    //     num2words::Num2Words::new(n)
    //         .to_words()
    //         .unwrap_or(caps[1].to_string())
    // });

    let mut result = text.to_string();

    // 2. Replace -- with em-dash
    result = result.replace("--", "—");

    // 3. Replace ... with ellipsis
    result = result.replace("...", "…");

    // todo: change this processing to proc macro

    // 4. Replace quotes manually
    // Double quotes
    let mut in_double = false;
    let mut final_text = String::with_capacity(result.len());
    for c in result.chars() {
        match c {
            '"' => {
                if in_double {
                    final_text.push('”'); // closing
                } else {
                    final_text.push('“'); // opening
                }
                in_double = !in_double;
            }
            '\'' => final_text.push('’'), // all single quotes to curly
            _ => final_text.push(c),
        }
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

// #[cfg(not(feature = "text"))]
//     pub fn linguate(input: &str) -> String {
//         input.to_string()
//     }

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
            result.push(buf.clone());
            buf.clear();
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

#[cfg(feature = "rand")]
pub fn find_hash_match<'a, I>(strings: I, target: u64) -> Option<&'a String>
where
    I: IntoIterator<Item = &'a String>,
{
    strings
        .into_iter()
        .find(|s| const_fnv1a_hash::fnv1a_hash_str_64(s) == target)
}

#[cfg(not(feature = "rand"))]
pub fn find_hash_match<'a, I>(strings: I, target: u64) -> Option<&'a String>
where
    I: IntoIterator<Item = &'a String>,
{
    strings.into_iter().nth(target as usize)
}
