use bar_module::{get_color, humanize, normalize};
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::{self, Error, ErrorKind, Read};
use std::path::PathBuf;
use std::thread;

use std::net::IpAddr;
use std::result::Result;
use std::time::{Duration, Instant};

struct Stat {
    rx: u64,
    tx: u64,
}

struct Connections {
    tcp4: u32,
    tcp6: u32,
    udp4: u32,
    udp6: u32,
}

fn get_ip(client: &reqwest::blocking::Client) -> Result<String, Error> {
    client
        .get("https://icanhazip.com/")
        .send()
        .map_err(|err| Error::new(ErrorKind::Other, err))
        .and_then(|r| r.text().map_err(|err| Error::new(ErrorKind::Other, err)))
        .and_then(|text| Ok(text.trim().to_string()))
}

fn is_ula(address: &IpAddr) -> bool {
    let ip6;
    match address {
        IpAddr::V6(ip) => ip6 = ip,
        _ => panic!(""),
    }
    let segments = ip6.segments();
    let first_segment = segments[0];
    // ULA addresses start with '1111 110' in the first 16-bit block
    (first_segment & 0xfe00) == 0xfc00
}

fn get_local_ips() -> (Option<IpAddr>, Option<IpAddr>) {
    let ips = if_addrs::get_if_addrs().unwrap();
    let ipv4s: Vec<IpAddr> = ips
        .iter()
        .filter(|ip| !ip.is_link_local() && !ip.is_loopback())
        .map(|ip| ip.addr.ip())
        .filter(|ip| ip.is_ipv4())
        .collect();
    let ipv6s: Vec<IpAddr> = ips
        .iter()
        .filter(|ip| !ip.is_link_local() && !ip.is_loopback())
        .map(|ip| ip.addr.ip())
        .filter(|ip| ip.is_ipv6() && !is_ula(&ip))
        .collect();
    (ipv4s.first().copied(), ipv6s.first().copied())
}

fn get_num_connections(filepath: &str) -> u32 {
    let mut file = File::open(filepath).expect("");
    let mut contents = String::new();

    file.read_to_string(&mut contents).expect("");
    contents.lines().count() as u32 - 1
}

fn get_connections() -> Connections {
    Connections {
        tcp4: get_num_connections("/proc/net/tcp"),
        tcp6: get_num_connections("/proc/net/tcp6"),
        udp4: get_num_connections("/proc/net/udp"),
        udp6: get_num_connections("/proc/net/udp6"),
    }
}

fn get_reads_and_writes(statfile: PathBuf) -> io::Result<(String, Stat)> {
    let dev_name = statfile.file_stem().expect("");
    if dev_name.to_str().expect("") == "lo" {
        return Err(io::Error::new(ErrorKind::Other, "Not up"));
    }

    let mut os_path = statfile.clone();
    os_path.push("operstate");

    let mut rx_path = statfile.clone();
    rx_path.push("statistics");
    rx_path.push("rx_bytes");

    let mut tx_path = statfile.clone();
    tx_path.push("statistics");
    tx_path.push("tx_bytes");

    let mut os_file = File::open(os_path)?;
    let mut os_contents = String::new();
    os_file.read_to_string(&mut os_contents)?;

    let mut rx_file = File::open(rx_path)?;
    let mut rx_contents = String::new();
    rx_file.read_to_string(&mut rx_contents)?;

    let mut tx_file = File::open(tx_path)?;
    let mut tx_contents = String::new();
    tx_file.read_to_string(&mut tx_contents)?;

    if os_contents == "down\n" || (tx_contents == "0\n" && rx_contents == "0\n") {
        return Err(io::Error::new(ErrorKind::Other, "Not up"));
    }

    Ok((
        dev_name.to_str().expect("").to_owned(),
        Stat {
            rx: rx_contents.trim().parse::<u64>().expect(""),
            tx: tx_contents.trim().parse::<u64>().expect(""),
        },
    ))
}

fn get_numbers() -> HashMap<String, Stat> {
    let mut values: HashMap<String, Stat> = HashMap::new();
    let paths = read_dir("/sys/class/net").expect("");
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

fn get_symbol(devname: &String) -> char {
    if devname.starts_with("wlp") || devname.starts_with("wlan") {
        return '';
    } else if devname.starts_with("eth") || devname.starts_with("eno") || devname.starts_with("enp")
    {
        return '󰈁';
    } else if devname.starts_with("tun") {
        return '';
    } else if devname.starts_with("wg") {
        return '';
    } else if devname.starts_with("wwan") {
        return '󰢿';
    } else if devname.starts_with("br-") || devname == "docker0" {
        return '󰘘';
    } else if devname.starts_with("veth") {
        return '';
    } else {
        return ' ';
    }
}

fn format_text(key: &String, dictionary: &HashMap<String, Stat>) -> String {
    let value = dictionary.get(key).expect("");
    String::from(format!(
        "{}<span foreground='#{}'></span><span foreground='#{}'></span>",
        get_symbol(key),
        get_color(&normalize(&(value.rx as f32), &37500000.)),
        get_color(&normalize(&(value.tx as f32), &12500000.))
    ))
}

fn format_tooltip(key: &String, dictionary: &HashMap<String, Stat>) -> String {
    let value = dictionary.get(key).expect("");
    String::from(format!(
        "{} {}: <span foreground='#{}'>{}</span> <span foreground='#{}'>{}</span>",
        get_symbol(key),
        key,
        get_color(&normalize(&(value.rx as f32), &37500000.)),
        humanize(value.rx),
        get_color(&normalize(&(value.tx as f32), &12500000.)),
        humanize(value.tx)
    ))
}

fn get_diff(
    recent: HashMap<String, Stat>,
    last: &HashMap<String, Stat>,
    ip4: &String,
    ip6: &String,
) -> () {
    let mut diff: HashMap<String, Stat> = HashMap::new();
    for (last_key, last_value) in last.iter() {
        if let Some(recent_value) = recent.get(last_key) {
            diff.insert(
                String::from(last_key),
                Stat {
                    rx: last_value.rx - recent_value.rx,
                    tx: last_value.tx - recent_value.tx,
                },
            );
        }
    }

    let mut keys: Vec<String> = diff.keys().cloned().collect();
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
    let connections = get_connections();
    println!(
        "{{\"text\": \"{}\", \"tooltip\": \"{}\r\rIPv4: {}\rIPv6: {}\r\rTCP: {}\rTCP6: {}\r\rUDP: {}\rUDP6: {}\"}}",
        text, tooltip, ip4, ip6, connections.tcp4, connections.tcp6, connections.udp4, connections.udp6
    );
}

fn main() {
    let sleeptime = Duration::from_secs(1);
    let mut old: HashMap<String, Stat> = HashMap::new();

    let ip_interval = Duration::new(600, 0);

    let mut start: Option<Instant> = None;
    let mut ipv4 = "".to_string();
    let mut ipv6 = "".to_string();
    loop {
        if start.is_none() || start.unwrap_or(Instant::now()).elapsed() > ip_interval {
            let (local4, local6) = get_local_ips();
            let client4 = local4.map(|ip| {
                reqwest::blocking::Client::builder()
                    .local_address(ip)
                    .build()
                    .unwrap()
            });
            let client6 = local6.map(|ip| {
                reqwest::blocking::Client::builder()
                    .local_address(ip)
                    .build()
                    .unwrap()
            });

            ipv4 = client4
                .as_ref()
                .map(|client| get_ip(&client).unwrap_or("".to_string()))
                .unwrap_or("".to_string());

            ipv6 = client6
                .as_ref()
                .map(|client| get_ip(&client).unwrap_or("".to_string()))
                .unwrap_or("".to_string());
            start = Some(Instant::now());
        }
        thread::sleep(sleeptime);
        let current = get_numbers();
        get_diff(old, &current, &ipv4, &ipv6);
        old = current;
    }
}
