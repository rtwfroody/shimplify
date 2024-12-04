use std::{collections::HashSet, io::Read};

#[derive(Debug)]
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
        loop {
            self.attempt += 1;
            if let Some(name) = match self.attempt {
                1 => self.text.last_part()
                    .and_then(|part| part.shorten(|c| c == '-')),
                2 => self.text.last_part()
                    .and_then(|part| part.shorten(|c| !c.is_alphanumeric())),
                3 => self.text.last_part()
                    .and_then(|part| part.shorten(|c| c == '/')),
                4 => self.text.last_part(),
                5 => self.text.shorten(|c| c == '/'),
                _ => Some(format!("{}_{}", self.text.shorten(|c| c == '/')
                        .unwrap_or("WTF".to_string()), self.attempt - 3)),
            } {
                return Some(name);
            }
        }
    }
}

fn legalize(var_name: &str) -> String {
    let var_name: String = var_name
        .to_uppercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if var_name.chars().next().unwrap().is_numeric() {
        format!("_{}", var_name)
    } else {
        var_name
    }
}

trait Shorten {
    fn shorten<F>(&self, separator: F) -> Option<String>
    where F: Fn(char) -> bool;
}

impl Shorten for str {
    fn shorten<F>(&self, separator: F) -> Option<String>
    where F: Fn(char) -> bool
    {
        let mut var_name = String::new();
        let mut add_char = true;
        for c in self.chars() {
            if add_char && c.is_alphanumeric() {
                var_name.push(c);
                add_char = false;
            }
            if separator(c) {
                add_char = true;
            }
        }
        if var_name.len() == 0 {
            None
        } else {
            Some(legalize(var_name.as_str()))
        }
    }
}

trait LastPart {
    fn last_part(&self) -> Option<String>;
}

impl LastPart for str {
    fn last_part(&self) -> Option<String> {
        // Get penultimate and last parts of the path.
        //println!("text: {}", self);
        match self.split('/')
                .filter(|part| !part.is_empty())
                .last() {
            Some(part) => Some(legalize(part)),
            None => None,
        }
    }
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

fn split_points(text: &str) -> Vec<usize> {
    let mut points = vec![0];
    let mut add_point = false;
    for (i, c) in text.chars().enumerate() {
        if add_point && c.is_alphanumeric() {
            points.push(i);
            add_point = false;
        }
        if !c.is_alphanumeric() {
            add_point = true;
        }
    }
    points.push(text.len());
    points
}

fn build_savings_table(used: &HashSet<String>, command: &str) -> Vec<(String, i32)> {
    // Divide into parts that make logical sense to a person to replace with a
    // variable. Breaking up "words" doesn't make sense.
    let parts = command.split(
        |c: char|
            c.is_whitespace()
            || c == ';'
            || c == '\''
            || c == '"'
        )
        // Ignore the LHS of assignments.
        .map(|part| part.split('=').last().unwrap())
        .filter(|part| !part.is_empty());

    // Make a frequency table of all subpaths in each path.
    let mut frequency_table = std::collections::HashMap::new();
    for part in parts {
        //println!("part: {}", part);
        let split_points = split_points(part);
        //println!("split_points: {:?}", split_points);
        for i in 0..split_points.len() {
            for j in i+1..split_points.len() {
                let subpath = &part[split_points[i]..split_points[j]];
                //println!("subpath: {}", subpath);
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
        let var_name = var_name(used, subpath);
        // Characters saved in the command by replacing this path.
        // Account for the $ sign as well as { and }
        let savings = (subpath.len() as i32 - var_name.len() as i32 - 3) * count;
        savings_table.push((subpath.to_string(), savings));
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
