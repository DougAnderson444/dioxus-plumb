/// RankDir represents the direction of graph layout.
/// It corresponds to the 'rankdir' attribute in DOT language.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RankDir {
    TB, // Top to Bottom (default)
    LR, // Left to Right
    BT, // Bottom to Top
    RL, // Right to Left
}

impl RankDir {
    // Associated constants for CSS classes
    pub(crate) const fn flex_class(self) -> &'static str {
        match self {
            RankDir::TB => "flex-col",
            RankDir::LR => "flex-row",
            RankDir::BT => "flex-col-reverse",
            RankDir::RL => "flex-row-reverse",
        }
    }

    // Parse from string
    pub(crate) fn from_str(s: &str) -> Self {
        match s.trim_matches('"') {
            "LR" => RankDir::LR,
            "BT" => RankDir::BT,
            "RL" => RankDir::RL,
            _ => RankDir::TB, // Default to TB
        }
    }

    // Convert to string for DOT output
    pub(crate) fn to_str(self) -> &'static str {
        match self {
            RankDir::TB => "TB",
            RankDir::LR => "LR",
            RankDir::BT => "BT",
            RankDir::RL => "RL",
        }
    }
}
