use bar_module::get_color;
use std::fs::File;
use std::io::{self, Read};
use std::thread;
use std::time::Duration;

fn get_numbers() -> io::Result<(Vec<i64>, Vec<i64>)> {
    let mut file = File::open("/proc/stat")?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    let relevant_lines: Vec<&str> = contents
        .lines()
        .filter(|line| {
            line.char_indices()
                .nth(3)
                .map_or(false, |(_, c)| c.is_digit(10))
        })
        .collect();
    let numbers: Vec<Vec<i64>> = relevant_lines
        .iter()
        .map(|line| {
            line.split_whitespace()
                .skip(1)
                .map(|s| s.parse::<i64>().expect("Failed parsing"))
                .collect()
        })
        .collect();
    let idle_sums: Vec<i64> = numbers.iter().map(|cpu| cpu[3] + cpu[4]).collect();
    let total_sums: Vec<i64> = numbers.iter().map(|cpu| cpu.iter().sum()).collect();

    Ok((idle_sums, total_sums))
}

fn output_difference(
    maybe_old: Option<(Vec<i64>, Vec<i64>)>,
    maybe_recent: Option<(Vec<i64>, Vec<i64>)>,
) -> () {
    if let Some(old) = maybe_old {
        if let Some(recent) = maybe_recent {
            let idle_difference: Vec<i64> =
                old.0.iter().zip(recent.0).map(|(o, n)| n - o).collect();
            let total_difference: Vec<i64> =
                old.1.iter().zip(recent.1).map(|(o, n)| n - o).collect();
            let differences: Vec<f32> = idle_difference
                .iter()
                .zip(total_difference)
                .map(|(i, t)| 1. - (*i as f32 / t as f32))
                .collect();
            let chars: String = differences
                .iter()
                .map(|v| String::from(format!("<span foreground='#{}'>●</span>", get_color(v))))
                .collect::<String>();
            println!(" {}", chars);
        }
    }
}

fn main() -> ! {
    let mut old: Option<(Vec<i64>, Vec<i64>)> = None;
    let sleeptime = Duration::from_secs(1);
    loop {
        thread::sleep(sleeptime);

        let recent = get_numbers().expect("");
        output_difference(old, Some(recent.clone()));
        old = Some(recent);
    }
}
