//! Per-window live publication topics and dirty masks.

use crate::{
    WINDOW_GAME_OVERLAY_LABEL, WINDOW_LIVE_LABEL, WINDOW_MAIN_LABEL, WINDOW_MINIMAP_OVERLAY_LABEL,
    WINDOW_MONSTER_OVERLAY_LABEL,
};

/// Bitmask of dirty publication topics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TopicMask(u8);

impl TopicMask {
    pub const EMPTY: Self = Self(0);
    pub const COMBAT: Self = Self(1 << 0);
    pub const STATUS: Self = Self(1 << 1);
    pub const BUFFS: Self = Self(1 << 2);
    pub const MONSTER: Self = Self(1 << 3);
    pub const FANTASY: Self = Self(1 << 4);
    pub const MINIMAP: Self = Self(1 << 5);
    pub const DEATHS: Self = Self(1 << 6);
    pub const SCENE: Self = Self(1 << 7);

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    /// Iterate set topic bits as named topics.
    pub fn iter(self) -> impl Iterator<Item = Topic> {
        Topic::ALL
            .into_iter()
            .filter(move |topic| self.contains(topic.mask()))
    }
}

impl std::ops::BitOr for TopicMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for TopicMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.insert(rhs);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    Combat,
    Status,
    Buffs,
    Monster,
    Fantasy,
    Minimap,
    Deaths,
    Scene,
}

impl Topic {
    /// Every topic, in the order used to index per-topic publication state.
    pub const ALL: [Self; Self::COUNT] = [
        Self::Combat,
        Self::Status,
        Self::Buffs,
        Self::Monster,
        Self::Fantasy,
        Self::Minimap,
        Self::Deaths,
        Self::Scene,
    ];

    pub const COUNT: usize = 8;

    /// Position of this topic in [`Topic::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Combat => 0,
            Self::Status => 1,
            Self::Buffs => 2,
            Self::Monster => 3,
            Self::Fantasy => 4,
            Self::Minimap => 5,
            Self::Deaths => 6,
            Self::Scene => 7,
        }
    }

    #[must_use]
    pub const fn mask(self) -> TopicMask {
        match self {
            Self::Combat => TopicMask::COMBAT,
            Self::Status => TopicMask::STATUS,
            Self::Buffs => TopicMask::BUFFS,
            Self::Monster => TopicMask::MONSTER,
            Self::Fantasy => TopicMask::FANTASY,
            Self::Minimap => TopicMask::MINIMAP,
            Self::Deaths => TopicMask::DEATHS,
            Self::Scene => TopicMask::SCENE,
        }
    }

    #[must_use]
    pub const fn event_name(self) -> &'static str {
        match self {
            Self::Combat => "live-combat",
            Self::Status => "live-status",
            Self::Buffs => "live-buffs",
            Self::Monster => "live-monster",
            Self::Fantasy => "live-fantasy",
            Self::Minimap => "minimap-snapshot",
            Self::Deaths => "live-deaths",
            Self::Scene => "live-scene",
        }
    }

    /// Target window labels for directed emit. Empty means broadcast (unused).
    #[must_use]
    pub const fn window_labels(self) -> &'static [&'static str] {
        match self {
            Self::Combat => &[WINDOW_LIVE_LABEL],
            Self::Status | Self::Buffs => &[WINDOW_GAME_OVERLAY_LABEL],
            Self::Monster => &[WINDOW_MONSTER_OVERLAY_LABEL],
            Self::Fantasy => &[WINDOW_LIVE_LABEL, WINDOW_MONSTER_OVERLAY_LABEL],
            Self::Minimap => &[WINDOW_MINIMAP_OVERLAY_LABEL],
            Self::Deaths => &[WINDOW_LIVE_LABEL],
            // The only consumer is the daily-scene auto-hide logic for the
            // game/monster/minimap overlay windows, which runs in `main`.
            Self::Scene => &[WINDOW_MAIN_LABEL],
        }
    }

    #[must_use]
    pub const fn is_immediate(self) -> bool {
        matches!(self, Self::Status)
    }

    #[must_use]
    pub const fn throttle_ms(self) -> Option<u64> {
        match self {
            Self::Combat => None, // uses configured event_update_rate_ms
            Self::Status => None, // per-batch immediate
            Self::Buffs | Self::Monster | Self::Fantasy | Self::Minimap | Self::Deaths
            | Self::Scene => Some(50),
        }
    }
}
