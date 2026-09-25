pub fn art_lines(condition: &str) -> [&'static str; 3] {
    let c = condition.to_ascii_lowercase();
    if contains_any(&c, &["thunder", "storm", "lightning"]) {
        ["     .--.    ", "   .( ** ).  ", "  / / / / /  "]
    } else if contains_any(&c, &["snow", "sleet", "blizzard"]) {
        ["    *  .  *  ", "  (    .   ) ", " *   '  *  ' "]
    } else if contains_any(&c, &["rain", "drizzle", "shower"]) {
        ["     , -- ._ ", "    (    ,~ )", " / ', \"/ '   "]
    } else if contains_any(&c, &["fog", "mist", "haze"]) {
        ["  ~~~~~~~~~~~", "   ~~~~~~~~~ ", "  ~~~~~~~~~~~"]
    } else if contains_any(&c, &["cloud", "overcast", "grey", "gray"]) {
        ["      .--.   ", "    .(    ). ", "   (___.___) "]
    } else if contains_any(&c, &["sun", "clear", "fair"]) {
        ["    \\  |  /  ", "  -  ( o )  -", "    /  |  \\  "]
    } else {
        ["      .--.   ", "    .(    ). ", "   (___.___) "]
    }
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

pub fn format_temp(temp_c: &str) -> String {
    let t = temp_c.trim();
    if t.is_empty() {
        return String::from("?");
    }
    if t.ends_with('C') || t.ends_with('c') {
        t.to_string()
    } else {
        format!("{t}C")
    }
}
