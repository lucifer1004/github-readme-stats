use chrono::{DateTime, Datelike, FixedOffset, Timelike};

use crate::models::TimeDistribution;

use super::commit_search::CommitSample;

/// Compute commit time distribution from a commit sample.
/// Returns a 24×7 grid of commit counts by hour and weekday.
pub(crate) fn compute_time_distribution(
    commits: &[CommitSample],
    timezone_offset: FixedOffset,
) -> TimeDistribution {
    let tz_str = format!(
        "{:+03}:{:02}",
        timezone_offset.local_minus_utc() / 3600,
        (timezone_offset.local_minus_utc().abs() % 3600) / 60
    );
    let mut distribution = TimeDistribution::new(tz_str);

    // Track earliest and latest commit dates
    let mut earliest: Option<DateTime<FixedOffset>> = None;
    let mut latest: Option<DateTime<FixedOffset>> = None;

    for commit in commits {
        let Some(utc_time) = commit.authored_at else {
            continue;
        };
        let local_time = utc_time.with_timezone(&timezone_offset);
        let hour = local_time.hour() as u8;
        // weekday(): Mon=0, Tue=1, ..., Sun=6
        let weekday = local_time.weekday().num_days_from_monday() as u8;
        distribution.add(hour, weekday);

        // Track date range
        match earliest {
            None => earliest = Some(local_time),
            Some(e) if local_time < e => earliest = Some(local_time),
            _ => {}
        }
        match latest {
            None => latest = Some(local_time),
            Some(l) if local_time > l => latest = Some(local_time),
            _ => {}
        }
    }

    // Set date range
    distribution.earliest_date = earliest.map(|d| d.format("%Y-%m-%d").to_string());
    distribution.latest_date = latest.map(|d| d.format("%Y-%m-%d").to_string());

    distribution.finalize();
    distribution
}
