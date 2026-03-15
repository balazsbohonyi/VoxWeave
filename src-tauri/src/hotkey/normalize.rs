const MODIFIER_ORDER: [&str; 4] = ["Ctrl", "Shift", "Alt", "Super"];

pub fn normalize_hotkey(input: &str) -> String {
    let mut modifiers = Vec::new();
    let mut key: Option<String> = None;

    for raw in input.split('+') {
        let token = raw.trim();
        if token.is_empty() {
            continue;
        }

        if let Some(modifier) = normalize_modifier(token) {
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
            continue;
        }

        key = Some(normalize_key(token));
    }

    let mut ordered = Vec::new();
    for modifier in MODIFIER_ORDER {
        if modifiers.iter().any(|m| m == modifier) {
            ordered.push(modifier.to_string());
        }
    }

    if let Some(key) = key {
        ordered.push(key);
    }

    ordered.join("+")
}

fn normalize_modifier(token: &str) -> Option<String> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some("Ctrl".to_string()),
        "shift" => Some("Shift".to_string()),
        "alt" | "option" => Some("Alt".to_string()),
        "super" | "meta" | "cmd" | "command" | "win" | "windows" => Some("Super".to_string()),
        _ => None,
    }
}

fn normalize_key(token: &str) -> String {
    let lower = token.trim().to_ascii_lowercase();
    match lower.as_str() {
        "space" => "Space".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "tab" => "Tab".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "pageup" => "PageUp".to_string(),
        "pagedown" => "PageDown".to_string(),
        _ => {
            if lower.len() == 1 {
                return lower.to_uppercase();
            }
            if lower.starts_with('f') && lower[1..].chars().all(|c| c.is_ascii_digit()) {
                return format!("F{}", &lower[1..]);
            }
            let mut chars = lower.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.collect::<String>()
                ),
                None => String::new(),
            }
        }
    }
}
