use enum_map::{Enum, EnumMap};

use crate::Class;

// TODO Come up with better shorthand names
// Note the Full parking ban case isn't captured here -- under that scenario, every road is
// effectively green
#[derive(Clone, Copy, Debug, PartialEq, Enum)]
pub enum Scenario {
    /// No parking restrictions
    U,
    /// Two-way traffic, parking one side only
    X,
    /// One-way traffic, parking both sies
    Y,
    /// One-way traffic, parking one side only
    Z,
}

#[derive(Clone, Copy, PartialEq, Enum)]
pub enum Rating {
    Red,
    Amber,
    Green,
}

impl Rating {
    pub fn new(scenario: Scenario, class: Class, width: f64) -> Self {
        // From an internal PDF
        let (desirable_min, absolute_min) = match (scenario, class) {
            // TODO Not filled out
            (_, Class::A) => (1.0, 0.0),

            (Scenario::U, Class::B) => (12.8, 11.5),
            (Scenario::X, Class::B) => (10.3, 9.0),
            (Scenario::Y, Class::B) => (8.9, 8.25),
            (Scenario::Z, Class::B) => (6.4, 5.75),

            (Scenario::U, Class::C) => (10.0, 9.1),
            (Scenario::X, Class::C) => (8.0, 7.3),
            (Scenario::Y, Class::C) => (7.0, 6.35),
            (Scenario::Z, Class::C) => (5.0, 4.55),

            (Scenario::U, Class::Unclassified) => (9.5, 8.4),
            (Scenario::X, Class::Unclassified) => (7.5, 6.6),
            (Scenario::Y, Class::Unclassified) => (6.75, 6.0),
            (Scenario::Z, Class::Unclassified) => (4.75, 4.2),
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

/// Isomorphic to `Option<Scenario>`, but seems more clear to represent this way.
#[derive(Clone, Copy, Debug, PartialEq, Enum)]
pub enum Intervention {
    /// No change needed (Scenario::U)
    None,
    Y,
    X,
    Z,
    /// No scenario yields green
    Impossible,
}

impl Intervention {
    // TODO The ordering here may need to vary locally, especially depending on the total
    // supply/demand of parking.
    pub fn calculate(ratings: &EnumMap<Scenario, Rating>, already_one_way: bool) -> Self {
        for (scenario, intervention) in [
            (Scenario::U, Intervention::None),
            (Scenario::Y, Intervention::Y),
            (Scenario::X, Intervention::X),
            (Scenario::Z, Intervention::Z),
        ] {
            if already_one_way && scenario == Scenario::X {
                continue;
            }

            if ratings[scenario] == Rating::Green {
                return intervention;
            }
        }
        Intervention::Impossible
    }
}
