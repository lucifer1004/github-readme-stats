use serde::Serialize;

/// Time distribution of commits across hours and weekdays
/// Grid is 24 hours × 7 weekdays (Mon=0, Sun=6)
#[derive(Debug, Clone, Serialize)]
pub struct TimeDistribution {
    /// 24×7 grid: grid[hour][weekday] = commit count
    pub grid: [[u32; 7]; 24],
    pub total_commits: u64,
    pub peak_hour: u8,
    pub peak_weekday: u8,
    pub timezone: String,
    /// Earliest commit date in the sample (YYYY-MM-DD)
    pub earliest_date: Option<String>,
    /// Latest commit date in the sample (YYYY-MM-DD)
    pub latest_date: Option<String>,
}

impl TimeDistribution {
    pub fn new(timezone: String) -> Self {
        Self {
            grid: [[0; 7]; 24],
            total_commits: 0,
            peak_hour: 0,
            peak_weekday: 0,
            timezone,
            earliest_date: None,
            latest_date: None,
        }
    }

    /// Add a commit at the given hour (0-23) and weekday (0=Mon, 6=Sun)
    pub fn add(&mut self, hour: u8, weekday: u8) {
        if hour < 24 && weekday < 7 {
            self.grid[hour as usize][weekday as usize] += 1;
            self.total_commits += 1;
        }
    }

    /// Compute peak hour and weekday after all commits are added
    pub fn finalize(&mut self) {
        let mut max_count = 0u32;
        let mut peak_h = 0u8;
        let mut peak_w = 0u8;

        for (h, row) in self.grid.iter().enumerate() {
            for (w, &count) in row.iter().enumerate() {
                if count > max_count {
                    max_count = count;
                    peak_h = h as u8;
                    peak_w = w as u8;
                }
            }
        }

        self.peak_hour = peak_h;
        self.peak_weekday = peak_w;
    }
}
