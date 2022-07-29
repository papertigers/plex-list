use console::style;
use std::io::Write;
use std::ops::{Div, Rem};
use unicode_truncate::{Alignment, UnicodeTruncateStr};
use unicode_width::UnicodeWidthStr;

use crate::plexpy::{PlexSession, PlexpyActivityData, PlexpyData, PlexpyHistoryData, SessionType};

const VERTICAL_LINE: &str = "│";
const HORIZONTAL_LINE: &str = "─";
const MUSIC: &str = "🎵";
const TV: &str = "📺";
const MOVIE: &str = "🎞 ";
const UNKNOWN: &str = "? ";
const SECURE: &str = "🔒";
const INSECURE: &str = "🔓";

/// Return the quotient and remainder between two T's.
/// Panics if rhs is zero.
fn div_rem<T>(lhs: T, rhs: T) -> (T, T)
where
    T: Div<Output = T> + Rem<Output = T> + Copy,
{
    (lhs / rhs, lhs % rhs)
}

/// Formats n seconds as days, hours, minutes and seconds.
pub fn pretty_duration(s: u64) -> String {
    let (days, s) = div_rem(s, 86_400);
    let (hours, s) = div_rem(s, 3600);
    let (minutes, s) = div_rem(s, 60);
    let seconds = s;
    if days > 0 {
        format!("{}d{}h{}m{:.1}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h{}m{:.1}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{:.1}s", minutes, seconds)
    } else {
        format!("{:.1}s", seconds)
    }
}

enum Style {
    Top,
    Bottom,
}

fn print_horizontal_line<W: Write>(
    handle: &mut W,
    style: Style,
    len: usize,
) -> std::io::Result<()> {
    match style {
        Style::Top => {
            writeln!(handle, "┌{}┐", HORIZONTAL_LINE.repeat(len))?;
        }
        Style::Bottom => {
            writeln!(handle, "└{}┘", HORIZONTAL_LINE.repeat(len))?;
        }
    }
    Ok(())
}

/// Generate a progress bar that represents the progress on a scale between 0 and 10
/// Panics if p is not between 0 and 100.
fn progress_bar(p: u8) -> String {
    assert!(p <= 100, "p must be between 0 and 100");
    let p = (p / 10) as usize;
    let pre = 10 - (10 - p);
    let post = 10 - p;
    format!(
        "{}{}{}{}{}",
        '[',
        HORIZONTAL_LINE.repeat(pre),
        '◼',
        HORIZONTAL_LINE.repeat(post),
        ']'
    )
}

fn is_secure(t: &Option<SessionType>) -> console::StyledObject<&str> {
    match t {
        Some(SessionType::Secure) => style(SECURE).green(),
        Some(SessionType::Insecure) => style(INSECURE).red(),
        None => style(""),
    }
}

fn print_session<W: Write>(mut handle: &mut W, session: &PlexSession) -> std::io::Result<()> {
    let media_type = session.media_type.to_lowercase();
    let media_type = match media_type.as_ref() {
        "track" => MUSIC,
        "movie" => MOVIE,
        "episode" => TV,
        _ => UNKNOWN,
    };
    // len = title + media glyph + number of spaces
    let len: usize = session.full_title.width() + media_type.width() + 3;

    let progress = session.progress_percent.parse().unwrap_or(0_u8);

    print_horizontal_line(&mut handle, Style::Top, len)?;
    writeln!(
        handle,
        "{} {} {} {}",
        VERTICAL_LINE,
        media_type,
        style(&session.full_title).bold().underlined(),
        VERTICAL_LINE
    )?;
    print_horizontal_line(&mut handle, Style::Bottom, len)?;
    writeln!(
        handle,
        "    {}: {}",
        style("State").italic().dim(),
        session.state
    )?;
    writeln!(
        handle,
        "    {}: {}",
        style("User").italic().dim(),
        session.user
    )?;
    writeln!(
        handle,
        "    {}: {} ({})",
        style("Player").italic().dim(),
        session.player,
        session.platform
    )?;
    writeln!(
        handle,
        "    {}: {}",
        style("Quality").italic().dim(),
        session.quality_profile
    )?;
    if !&session.transcode_container.is_empty() {
        writeln!(
            handle,
            "    {}: {} -> {}",
            style("Container").italic().dim(),
            session.container,
            session.transcode_container
        )?;
    } else {
        writeln!(
            handle,
            "    {}: {}",
            style("Container").italic().dim(),
            session.container
        )?;
    }
    writeln!(
        handle,
        "    {}: {}",
        style("Progress").italic().dim(),
        progress_bar(progress)
    )?;
    writeln!(
        handle,
        "    {}: {}{}",
        style("IP").italic().dim(),
        is_secure(&session.session_type),
        session.ip_address_public
    )?;
    writeln!(handle, "\n")?;
    Ok(())
}

fn print_sessions<W: Write>(mut handle: W, data: &PlexpyActivityData) -> std::io::Result<()> {
    for session in &data.sessions {
        print_session(&mut handle, session)?;
    }
    Ok(())
}

fn print_history<W: Write>(mut handle: W, data: &PlexpyHistoryData) -> std::io::Result<()> {
    writeln!(
        handle,
        "{:^60}{:>20}{:>20}",
        style("Media").bold().underlined(),
        style("User").bold().underlined(),
        style("Duration").bold().underlined(),
    )?;
    for entry in &data.history {
        writeln!(
            handle,
            "{} {} {:>20}",
            &entry.full_title.unicode_pad(58, Alignment::Left, true),
            &entry.user.unicode_pad(20, Alignment::Right, true),
            pretty_duration(entry.duration),
        )?;
    }
    Ok(())
}

pub fn print_data<W: Write>(mut handle: W, data: &PlexpyData) -> std::io::Result<()> {
    match data {
        PlexpyData::Activity(activity) => print_sessions(&mut handle, activity),
        PlexpyData::History(history) => print_history(&mut handle, history),
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_progress_bar() {
        use super::progress_bar;
        let none = "[◼──────────]";
        let half = "[─────◼─────]";
        let full = "[──────────◼]";

        assert_eq!(none, progress_bar(0), "progress 0%");
        assert_eq!(half, progress_bar(50), "progress 50%");
        assert_eq!(full, progress_bar(100), "progress 100%");
    }

    #[test]
    #[should_panic(expected = "p must be between 0 and 100")]
    fn test_progress_bar_invalid_input() {
        use super::progress_bar;
        let _ = progress_bar(255);
    }

    #[test]
    #[should_panic]
    fn div_rem_zero() {
        let _ = div_rem(100, 0);
    }

    #[test]
    fn div_rem_nonzero() {
        assert_eq!(div_rem(5, 5), (1, 0));
        assert_eq!(div_rem(10, 5), (2, 0));
        assert_eq!(div_rem(3, 5), (0, 3));
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(pretty_duration(60), "1m0s");
        assert_eq!(pretty_duration(3600), "1h0m0s");
        assert_eq!(pretty_duration(86_400), "1d0h0m0s");
        assert_eq!(pretty_duration(1242), "20m42s");
        assert_eq!(pretty_duration(20231), "5h37m11s");
    }
}
