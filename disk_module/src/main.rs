use bar_module::{get_color, humanize, normalize};
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

struct Stat {
    writes: u64,
    reads: u64,
}

fn get_reads_and_writes(statfile: PathBuf) -> io::Result<(String, Stat)> {
    let diskname = statfile.file_stem().expect("");
    let mut statfile = statfile.clone();
    statfile.push("stat");

    let mut file = File::open(statfile)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let numbers: Vec<u64> = contents
        .split_whitespace()
        .map(|s| s.parse::<u64>().expect(""))
        .collect();
    Ok((
        diskname.to_str().expect("").to_owned(),
        Stat {
            reads: numbers[2] * 512,
            writes: numbers[6] * 512,
        },
    ))
}

fn get_numbers() -> HashMap<String, Stat> {
    let mut values: HashMap<String, Stat> = HashMap::new();
    let paths = read_dir("/sys/class/block").expect("");
    for path in paths {
        let stat: Option<(String, Stat)> = path
            .ok()
            .map(|val| get_reads_and_writes(val.path()).ok())
            .flatten();
        if let Some((diskname, stat)) = stat {
            values.insert(diskname, stat);
        }
    }
    values
}

fn format_text(key: &String, dictionary: &HashMap<String, Stat>) -> String {
    let value = dictionary.get(key).expect("");
    String::from(format!(
        "<span foreground='#{}'></span> <span foreground='#{}'></span>",
        get_color(&normalize(&(value.reads as f32), &100000000.)),
        get_color(&normalize(&(value.writes as f32), &100000000.))
    ))
}

fn format_tooltip(key: &String, dictionary: &HashMap<String, Stat>) -> String {
    let value = dictionary.get(key).expect("");
    String::from(format!(
        "{}: <span foreground='#{}'>{}</span><span foreground='#{}'>{}</span>",
        key,
        get_color(&normalize(&(value.reads as f32), &100000000.)),
        humanize(value.reads),
        get_color(&normalize(&(value.writes as f32), &100000000.)),
        humanize(value.writes)
    ))
}

fn get_diff(recent: HashMap<String, Stat>, last: &HashMap<String, Stat>) -> () {
    let mut diff: HashMap<String, Stat> = HashMap::new();
    for (last_key, last_value) in last.iter() {
        if let Some(recent_value) = recent.get(last_key) {
            diff.insert(
                String::from(last_key),
                Stat {
                    reads: last_value.reads - recent_value.reads,
                    writes: last_value.writes - recent_value.writes,
                },
            );
        }
    }

    let mut keys: Vec<String> = Vec::new();
    let all_keys: Vec<String> = diff.keys().cloned().collect();
    for outer in all_keys.iter() {
        let mut keep = true;
        for inner in all_keys.iter() {
            if outer != inner {
                if inner.starts_with(outer) || outer.starts_with("dm-") {
                    keep = false;
                }
            }
        }
        if keep {
            keys.push(String::from(outer));
        }
    }

    keys.sort();

    let text = keys
        .iter()
        .map(|k| format_text(k, &diff))
        .collect::<Vec<String>>()
        .join(" ");
    let tooltip = keys
        .iter()
        .map(|k| format_tooltip(k, &diff))
        .collect::<Vec<String>>()
        .join("\r");
    println!("{{\"text\": \" {}\", \"tooltip\": \"{}\"}}", text, tooltip);
}

fn main() {
    let sleeptime = Duration::from_secs(1);
    let mut old: HashMap<String, Stat> = HashMap::new();
    loop {
        thread::sleep(sleeptime);
        let current = get_numbers();
        get_diff(old, &current);
        old = current;
    }
}
