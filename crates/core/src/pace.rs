//! Pace (P-016): how fast a window is filling, and when it runs out if that comes before it resets.
//!
//! The daemon keeps every reading of every window in its history. A window's pace is a straight
//! line fitted by least squares through its readings since it last reset, ending with the reading
//! on show. Until there are at least [`MIN_READINGS`] readings spanning [`MIN_SPAN`], there is too
//! little to go on, and the window has no pace.

use chrono::{DateTime, TimeDelta, Utc};

use crate::{Pace, Window};

/// Fewer readings than this give no pace.
pub const MIN_READINGS: usize = 3;
/// Readings spanning less time than this give no pace.
pub const MIN_SPAN: TimeDelta = TimeDelta::minutes(30);
/// A reading more than this many points below the one before it means the window started again:
/// it reset, or its provider corrected it. A smaller fall is rounding.
const FALL: f64 = 1.0;
/// A window that would run out further away than this is not running out.
const FURTHEST: TimeDelta = TimeDelta::days(366);

/// One reading of a window, from the daemon's history.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    /// When it was fetched.
    pub fetched_at: DateTime<Utc>,
    /// The share used then.
    pub used_pct: f64,
}

impl Pace {
    /// When `window` began, as its reset less its length, if both are known. No reading before
    /// then counts towards its pace.
    #[must_use]
    pub fn since(window: &Window) -> Option<DateTime<Utc>> {
        if window.window_seconds == 0 {
            return None;
        }
        let length = TimeDelta::try_seconds(i64::try_from(window.window_seconds).ok()?)?;
        window.resets_at?.checked_sub_signed(length)
    }

    /// `window`'s pace from its earlier `readings`, oldest first. The readings that count are those
    /// from before the window's own reading, since it began ([`Pace::since`]) and since its share
    /// used last fell by more than a point; the window's own reading ends them.
    ///
    /// There is no pace below [`MIN_READINGS`] readings over [`MIN_SPAN`], and none for a window that
    /// is already full or whose 0% is unconfirmed. A window that is not filling has a pace of 0.
    #[must_use]
    pub fn of(window: &Window, readings: &[Reading]) -> Option<Self> {
        if window.unconfirmed || window.used_pct >= 100.0 {
            return None;
        }
        let since = Self::since(window);
        let mut counted: Vec<Reading> = readings
            .iter()
            .filter(|reading| {
                reading.fetched_at < window.fetched_at
                    && since.is_none_or(|since| reading.fetched_at >= since)
            })
            .copied()
            .collect();
        counted.push(Reading {
            fetched_at: window.fetched_at,
            used_pct: window.used_pct,
        });
        if let Some(fall) = counted
            .windows(2)
            .rposition(|pair| pair[1].used_pct < pair[0].used_pct - FALL)
        {
            counted.drain(..=fall);
        }
        let span = window.fetched_at - counted[0].fetched_at;
        if counted.len() < MIN_READINGS || span < MIN_SPAN {
            return None;
        }
        // Usage never falls within a window, so a line that does is noise around no use at all.
        let pct_per_hour = (slope(&counted)?.max(0.0) * 100.0).round() / 100.0;
        Some(Self {
            pct_per_hour,
            runs_out_at: runs_out_at(window, pct_per_hour),
        })
    }
}

/// The least-squares slope through `readings`, in points per hour.
fn slope(readings: &[Reading]) -> Option<f64> {
    let start = readings.first()?.fetched_at;
    let points: Vec<(f64, f64)> = readings
        .iter()
        .map(|reading| {
            let hours = (reading.fetched_at - start).num_milliseconds() as f64 / 3_600_000.0;
            (hours, reading.used_pct)
        })
        .collect();
    let count = points.len() as f64;
    let mean_x = points.iter().map(|(x, _)| x).sum::<f64>() / count;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f64>() / count;
    let (mut sxx, mut sxy) = (0.0, 0.0);
    for (x, y) in &points {
        sxx += (x - mean_x) * (x - mean_x);
        sxy += (x - mean_x) * (y - mean_y);
    }
    let slope = sxy / sxx;
    slope.is_finite().then_some(slope)
}

/// When `window` reaches 100% at `pct_per_hour` from its own reading, if that comes before it
/// resets, to the second.
fn runs_out_at(window: &Window, pct_per_hour: f64) -> Option<DateTime<Utc>> {
    if pct_per_hour <= 0.0 {
        return None;
    }
    // A float too large for the cast saturates, and then fails the bound below.
    let seconds = ((100.0 - window.used_pct) / pct_per_hour * 3600.0).round() as i64;
    let left = TimeDelta::try_seconds(seconds).filter(|left| *left <= FURTHEST)?;
    let at = window.fetched_at.checked_add_signed(left)?;
    let at = DateTime::from_timestamp(at.timestamp(), 0)?;
    window
        .resets_at
        .is_none_or(|resets_at| at < resets_at)
        .then_some(at)
}

#[cfg(test)]
mod tests {
    use crate::{Source, WindowKind};

    use super::*;

    fn at(minutes: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
            .expect("valid timestamp")
            .with_timezone(&Utc)
            + TimeDelta::minutes(minutes)
    }

    /// A five-hour session that began at minute 0, read at `minute`.
    fn session(used_pct: f64, minute: i64) -> Window {
        Window {
            kind: WindowKind::Session,
            label: "Session".into(),
            used_pct,
            resets_at: Some(at(300)),
            window_seconds: 18_000,
            scope_model: None,
            source: Source::Api,
            fetched_at: at(minute),
            unconfirmed: false,
            pace: None,
        }
    }

    /// Readings at each `(minute, used_pct)`.
    fn readings(points: &[(i64, f64)]) -> Vec<Reading> {
        points
            .iter()
            .map(|(minute, used_pct)| Reading {
                fetched_at: at(*minute),
                used_pct: *used_pct,
            })
            .collect()
    }

    #[test]
    fn a_steady_climb_gives_its_rate_and_when_it_runs_out() {
        // 2 points every 10 minutes: 12 points an hour, from 40% at minute 60.
        let window = session(40.0, 60);
        let earlier = readings(&[
            (0, 28.0),
            (10, 30.0),
            (20, 32.0),
            (30, 34.0),
            (40, 36.0),
            (50, 38.0),
        ]);

        let pace = Pace::of(&window, &earlier).expect("enough readings");

        assert_eq!(pace.pct_per_hour, 12.0);
        // 60 points to go at 12 an hour is 5 hours, after the reset at minute 300.
        assert_eq!(pace.runs_out_at, None, "the session resets first");

        let window = session(70.0, 60);
        let earlier = readings(&[(0, 10.0), (20, 30.0), (40, 50.0)]);
        let pace = Pace::of(&window, &earlier).expect("enough readings");
        assert_eq!(pace.pct_per_hour, 60.0);
        assert_eq!(pace.runs_out_at, Some(at(90)), "30 points at 60 an hour");
    }

    #[test]
    fn there_is_no_pace_until_three_readings_span_half_an_hour() {
        let window = session(30.0, 30);

        assert_eq!(
            Pace::of(&window, &readings(&[(0, 10.0)])),
            None,
            "two readings"
        );
        assert_eq!(
            Pace::of(&session(30.0, 29), &readings(&[(0, 10.0), (15, 20.0)])),
            None,
            "29 minutes"
        );
        assert!(Pace::of(&window, &readings(&[(0, 10.0), (15, 20.0)])).is_some());
    }

    #[test]
    fn only_the_readings_since_the_window_began_count() {
        // The last session ran out at 100%, then nothing was used until this one began at minute 0.
        let window = session(30.0, 60);
        let earlier = readings(&[
            (-90, 96.0),
            (-60, 100.0),
            (-40, 0.0),
            (-20, 0.0),
            (0, 0.0),
            (30, 15.0),
        ]);

        let pace = Pace::of(&window, &earlier).expect("enough readings");

        assert_eq!(pace.pct_per_hour, 30.0);
    }

    #[test]
    fn without_a_reset_time_a_fall_starts_the_readings_again() {
        let window = Window {
            resets_at: None,
            ..session(20.0, 60)
        };
        let earlier = readings(&[(-20, 80.0), (0, 85.0), (20, 0.0), (40, 10.0)]);

        let pace = Pace::of(&window, &earlier).expect("enough readings");

        assert_eq!(pace.pct_per_hour, 30.0);
        assert_eq!(pace.runs_out_at, Some(at(220)), "no reset to beat");
    }

    #[test]
    fn a_fall_of_a_point_or_less_is_rounding() {
        let window = session(41.0, 40);
        let earlier = readings(&[(0, 40.0), (20, 39.0)]);

        let pace = Pace::of(&window, &earlier).expect("one series");

        assert_eq!(pace.pct_per_hour, 1.5);
    }

    #[test]
    fn a_window_that_is_not_filling_has_a_pace_of_zero() {
        let still = Pace::of(&session(40.0, 60), &readings(&[(0, 40.0), (30, 40.0)]));
        assert_eq!(
            still,
            Some(Pace {
                pct_per_hour: 0.0,
                runs_out_at: None
            })
        );

        // A line that falls by less than a point at a time is noise around no use.
        let falling = Pace::of(&session(39.2, 60), &readings(&[(0, 40.0), (30, 39.6)]));
        assert_eq!(falling.map(|pace| pace.pct_per_hour), Some(0.0));
    }

    #[test]
    fn a_full_or_unconfirmed_window_has_no_pace() {
        let earlier = readings(&[(0, 80.0), (30, 90.0)]);

        assert_eq!(Pace::of(&session(100.0, 60), &earlier), None);
        let unconfirmed = Window {
            unconfirmed: true,
            ..session(0.0, 60)
        };
        assert_eq!(Pace::of(&unconfirmed, &earlier), None);
    }

    #[test]
    fn the_window_on_show_ends_the_readings() {
        // A reading newer than the window shown, and one at the same time, are left out.
        let window = session(30.0, 60);
        let earlier = readings(&[(0, 10.0), (30, 20.0), (60, 99.0), (90, 99.0)]);

        let pace = Pace::of(&window, &earlier).expect("enough readings");

        assert_eq!(pace.pct_per_hour, 20.0);
    }

    #[test]
    fn a_run_out_more_than_a_year_away_is_none() {
        let window = Window {
            resets_at: None,
            ..session(10.0, 600)
        };
        let earlier = readings(&[(0, 9.9), (300, 9.95)]);

        let pace = Pace::of(&window, &earlier).expect("enough readings");

        // 90 points at 0.01 an hour is 375 days.
        assert_eq!(pace.pct_per_hour, 0.01);
        assert_eq!(pace.runs_out_at, None);

        let slower = Pace::of(&window, &readings(&[(0, 9.999), (300, 9.9995)]));
        assert_eq!(
            slower,
            Some(Pace {
                pct_per_hour: 0.0,
                runs_out_at: None
            }),
            "rounded to hundredths, and not filling"
        );
    }

    #[test]
    fn a_window_began_at_its_reset_less_its_length() {
        assert_eq!(Pace::since(&session(0.0, 0)), Some(at(0)));
        assert_eq!(
            Pace::since(&Window {
                resets_at: None,
                ..session(0.0, 0)
            }),
            None
        );
        assert_eq!(
            Pace::since(&Window {
                window_seconds: 0,
                ..session(0.0, 0)
            }),
            None,
            "a limit of unknown length"
        );
    }
}
