use std::fmt::Display;

pub enum Priority {
    None,
    Urgent,
    High,
    Normal,
    Low,
}

impl Display for Priority {
    /// This is for rendering in a select
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::None => write!(f, "None"),
            Priority::Urgent => write!(f, "Urgent"),
            Priority::Normal => write!(f, "Normal"),
            Priority::Low => write!(f, "Low"),
            Priority::High => write!(f, "High"),
        }
    }
}
pub fn priority_to_int(priority: &Priority) -> u8 {
    match priority {
        Priority::None => 0,
        Priority::Urgent => 1,
        Priority::High => 2,
        Priority::Normal => 3,
        Priority::Low => 4,
    }
}
pub fn all_priorities() -> Vec<Priority> {
    vec![
        Priority::Low,
        Priority::Normal,
        Priority::High,
        Priority::Urgent,
        Priority::None,
    ]
}
