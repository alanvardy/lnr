use colored::*;

pub fn green_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).green().to_string()
}

pub fn cyan_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).bright_cyan().to_string()
}

pub fn blue_string(str: &str) -> String {
    if cfg!(test) {
        return normal_string(str);
    }

    String::from(str).blue().to_string()
}

pub fn normal_string(str: &str) -> String {
    String::from(str).normal().to_string()
}
