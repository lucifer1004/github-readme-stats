use serde::Serialize;

/// Implements [[RFC-0002:C-CONTRIBUTION-CALENDAR]]
#[derive(Debug, Clone, Serialize)]
pub struct ContributionCalendar {
    pub total_contributions: u64,
    pub weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContributionWeek {
    pub days: Vec<ContributionDay>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContributionDay {
    pub date: String,
    pub contribution_count: u64,
    pub level: u8,
}

/// Implements [[RFC-0002:C-STREAK]]
#[derive(Debug, Clone, Serialize)]
pub struct StreakStats {
    pub current_streak: u64,
    pub longest_streak: u64,
    pub streak_start_date: Option<String>,
}

impl ContributionCalendar {
    /// Compute streak stats from calendar data
    pub fn compute_streaks(&self) -> StreakStats {
        let mut all_days: Vec<&ContributionDay> =
            self.weeks.iter().flat_map(|w| w.days.iter()).collect();

        // Sort by date ascending
        all_days.sort_by(|a, b| a.date.cmp(&b.date));

        if all_days.is_empty() {
            return StreakStats {
                current_streak: 0,
                longest_streak: 0,
                streak_start_date: None,
            };
        }

        // Compute longest streak
        let mut longest = 0u64;
        let mut current = 0u64;

        for day in &all_days {
            if day.contribution_count > 0 {
                current += 1;
                longest = longest.max(current);
            } else {
                current = 0;
            }
        }

        // Compute current streak (from the end, allowing today to be 0)
        let mut current_streak = 0u64;
        let mut streak_start: Option<&str> = None;
        let mut started = false;

        for day in all_days.iter().rev() {
            if day.contribution_count > 0 {
                current_streak += 1;
                streak_start = Some(&day.date);
                started = true;
            } else if started {
                // Streak broken
                break;
            }
            // If we haven't started and today is 0, keep looking back
        }

        StreakStats {
            current_streak,
            longest_streak: longest,
            streak_start_date: streak_start.map(String::from),
        }
    }
}
