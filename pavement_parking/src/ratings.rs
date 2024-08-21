use crate::Class;

#[derive(Clone, Copy, PartialEq)]
pub enum Rating {
    Red,
    Amber,
    Green,
}

impl Rating {
    pub fn new(class: Class, width: f64) -> Self {
        match class {
            Class::A | Class::B => {
                if width >= 11.8 {
                    Self::Green
                } else if width >= 10.4 {
                    Self::Amber
                } else {
                    Self::Red
                }
            }

            Class::C | Class::Unclassified => {
                if width >= 9.0 {
                    Self::Green
                } else if width >= 7.5 {
                    Self::Amber
                } else {
                    // TODO Table doesn't handle [7, 7.5]
                    Self::Red
                }
            }
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
