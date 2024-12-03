use std::{collections::HashSet, io::Read};

struct NameIter<'a> {
    text: &'a str,
    attempt: usize,
}

impl<'a> NameIter<'a> {
    fn new(text: &'a str) -> Self {
        NameIter {
            text,
            attempt: 0
        }
    }
}

impl Iterator for NameIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.attempt += 1;
        match self.attempt {
            1 => Some(shorten(last_part(self.text).as_str(), |c| c == '-')),
            2 => Some(shorten(last_part(self.text).as_str(), |c| !c.is_alphanumeric())),
            3 => Some(last_part(self.text)),
            4 => Some(shorten(self.text, |c| c == '/')),
            _ => format!("{}_{}", shorten(self.text, |c| c == '/'), self.attempt - 3).into(),
        }
    }
}

fn shorten<F>(text: &str, separator: F) -> String
where F: Fn(char) -> bool
{
    let mut var_name = String::new();
    let mut add_char = true;
    for c in text.chars() {
        if add_char && c.is_alphanumeric() {
            var_name.push(c);
            add_char = false;
        }
        if separator(c) {
            add_char = true;
        }
    }
    var_name.to_uppercase()
}

fn last_part(text: &str) -> String {
    text.split('/').last().unwrap().to_uppercase()
}

fn var_name(
        used: &HashSet<String>,
        subpath: &str) -> String {
    let name_iter = NameIter::new(subpath);
    for var_name in name_iter {
        if !used.contains(&var_name) {
            return var_name;
        }
    }
    panic!("No available variable name found for {}", subpath);
}

fn build_savings_table(used: &HashSet<String>, command: &str) -> Vec<(String, i32)> {
    // Find every path in the command.
    let paths: Vec<&str> = command
        .split(
            |c: char| c != '/' && c != ':' && c != '-' && c != '_' && c != '.' &&
                (c.is_whitespace() || c.is_ascii_punctuation())
        )
        .filter(|s| s.contains("/"))
        .collect();

    // Make a frequency table of all subpaths in each path.
    let mut frequency_table = std::collections::HashMap::new();
    for path in paths {
        let parts = path.split('/').collect::<Vec<_>>();
        for i in 0..parts.len() {
            for j in i..parts.len() {
                let subpath = parts[i..j+1].join("/");
                let count = frequency_table.entry(subpath).or_insert(0);
                *count += 1;
            }
        }
    }

    // Turn that into a savings table, which is the number of bytes saved by
    // replacing the subpath with a variable.
    let mut savings_table = Vec::new();
    for (subpath, count) in frequency_table.into_iter() {
        if count < 2 {
            continue;
        }
        let var_name = var_name(used, subpath.as_str());
        // Characters saved in the command by replacing this path.
        // Account for the $ sign, but not { and }
        let savings = (subpath.len() as i32 - var_name.len() as i32 - 1) * count;
        savings_table.push((subpath, savings));
    }
    savings_table.sort_by(|a, b| b.1.cmp(&a.1));
    savings_table
}

fn main() {
    // Read stdin into shell_command.
    let mut shell_command = Vec::new();
    std::io::stdin().read_to_end(&mut shell_command).unwrap();
    let mut shell_command = String::from_utf8(shell_command).unwrap();

    let mut used = HashSet::new();

    loop {
        let savings_table = build_savings_table(&used, &shell_command);
        if savings_table.is_empty() {
            break;
        }

        let (subpath, savings) = savings_table.get(0).unwrap();
        if *savings < 2 {
            break;
        }
        let var_name = var_name(&used, subpath);
        assert!(!used.contains(&var_name));
        used.insert(var_name.clone());

        println!("{}={}", var_name, subpath);
        shell_command = shell_command.replace(subpath, &format!("${{{}}}", var_name));
    }
    println!("{}", shell_command);
}
