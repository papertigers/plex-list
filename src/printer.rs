use console::style;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

use crate::plexpy::{PlexSession, PlexpyData, SessionType};

const VERTICAL_LINE: &str = "│";
const HORIZONTAL_LINE: &str = "─";
const MUSIC: &str = "🎵";
const TV: &str = "📺";
const MOVIE: &str = "🎞 ";
const UNKNOWN: &str = "? ";
const SECURE: &str = "🔒";
const INSECURE: &str = "🔓";

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
    let len: usize = UnicodeWidthStr::width(session.full_title.as_str())
        + UnicodeWidthStr::width(media_type)
        + 3;

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
        "    {}: {}",
        style("Player").italic().dim(),
        format!("{} ({})", session.player, session.platform)
    )?;
    writeln!(
        handle,
        "    {}: {}",
        style("Quality").italic().dim(),
        session.quality_profile
    )?;
    if &session.transcode_container != "" {
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

pub fn print_data<W: Write>(mut handle: W, data: &PlexpyData) -> std::io::Result<()> {
    for session in &data.sessions {
        print_session(&mut handle, session)?;
    }
    Ok(())
}

mod test {
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
}