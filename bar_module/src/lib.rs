pub fn get_color(value: &f32) -> String {
    String::from(format!("{}{}00", get_red(value), get_green(value)))
}

pub fn normalize(value: &f32, limit: &f32) -> f32 {
    1. - (-value / limit * 2.).exp()
}

pub fn humanize(bytes: u64) -> String {
    if bytes >= (1u64 << 40) {
        return format!("{:5.1}T", (bytes as f64) / ((1u64) << 40) as f64);
    }
    if bytes >= (1u64 << 30) {
        return format!("{:5.1}G", (bytes as f64) / ((1u64) << 30) as f64);
    }
    if bytes >= (1u64 << 20) {
        return format!("{:5.1}M", (bytes as f64) / ((1u64) << 20) as f64);
    }
    if bytes >= (1u64 << 10) {
        return format!("{:5.1}K", (bytes as f64) / ((1u64) << 10) as f64);
    }
    format!("{:5}B", bytes)
}

fn get_red(value: &f32) -> String {
    let red_value: f32;
    if *value < 0.5 {
        red_value = *value * 2.;
    } else {
        red_value = 1.;
    }
    String::from(format!("{:02X}", (red_value * 255.) as i32))
}

fn get_green(value: &f32) -> String {
    let green_value: f32;
    if *value < 0.5 {
        green_value = 1.;
    } else {
        green_value = (1. - *value) * 2.;
    }
    String::from(format!("{:02X}", (green_value * 255.) as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_color() {
        assert_eq!(get_color(&0.), "00FF00");
        assert_eq!(get_color(&1.), "FF0000");
        assert_eq!(get_color(&0.5), "FFFF00");
    }
}
