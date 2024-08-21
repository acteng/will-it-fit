use crate::Class;

#[derive(Clone, Copy, PartialEq)]
pub enum Rating {
    Red,
    Amber,
    Green,
}

impl Rating {
    pub fn new(class: Class, width: f64) -> Self {
        // From an internal PDF, the "No parking restriction" scenario
        let (desirable_min, absolute_min) = match class {
            // TODO Not filled out
            Class::A => (1.0, 0.0),
            Class::B => (12.8, 11.5),
            Class::C => (10.0, 9.1),
            Class::Unclassified => (9.5, 8.4),
        };

        if width >= desirable_min {
            Self::Green
        } else if width >= absolute_min {
            Self::Amber
        } else {
            Self::Red
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Amber => "amber",
            Self::Green => "green",
        }
    }
}

#[allow(unused)]
fn old_table(class: Class, width: f64) -> Rating {
    match class {
        Class::A | Class::B => {
            if width >= 11.8 {
                Rating::Green
            } else if width >= 10.4 {
                Rating::Amber
            } else {
                Rating::Red
            }
        }

        Class::C | Class::Unclassified => {
            if width >= 9.0 {
                Rating::Green
            } else if width >= 7.5 {
                Rating::Amber
            } else {
                // TODO Table doesn't handle [7, 7.5]
                Rating::Red
            }
        }
    }
}
