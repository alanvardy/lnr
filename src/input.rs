use std::fmt::Display;

use inquire::{Editor, Select, Text};

/// Get text input from user
pub fn string(desc: &str, mock_string: Option<String>) -> Result<String, String> {
    if cfg!(test) {
        if let Some(string) = mock_string {
            Ok(string)
        } else {
            panic!("Must set mock_string in config")
        }
    } else {
        Text::new(desc).prompt().map_err(|e| e.to_string())
    }
}

/// Get large amount of text from user using editor
pub fn editor(
    desc: &str,
    default_text: &str,
    mock_string: Option<String>,
) -> Result<String, String> {
    if cfg!(test) {
        if let Some(string) = mock_string {
            Ok(string)
        } else {
            panic!("Must set mock_string in config")
        }
    } else {
        Editor::new(desc)
            .with_predefined_text(default_text)
            .prompt()
            .map_err(|e| e.to_string())
    }
}

/// Select an input from a list
pub fn select<T: Display>(
    desc: &str,
    options: Vec<T>,
    mock_select: Option<usize>,
) -> Result<T, String> {
    if cfg!(test) {
        if let Some(index) = mock_select {
            Ok(options
                .into_iter()
                .nth(index)
                .expect("Must provide a vector of options"))
        } else {
            panic!("Must set mock_select in config")
        }
    } else {
        Select::new(desc, options)
            .prompt()
            .map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_select() {
        let result = select("type", vec!["there", "are", "words"], Some(0));
        let expected = Ok("there");
        assert_eq!(result, expected);

        let result = select("type", vec!["there", "are", "words"], Some(1));
        let expected = Ok("are");
        assert_eq!(result, expected);
    }
}
