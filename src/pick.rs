//! Chooses which app a letter press should bring to the front.

/// A running app the user can switch to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub pid: i32,
    pub name: String,
}

/// Returns the app to focus when the user presses `letter`.
///
/// Apps whose name starts with `letter` are taken in name order. If the
/// frontmost app is one of them, the next one is chosen, so repeated presses
/// cycle. Otherwise the first one is chosen.
pub fn pick(apps: &[App], letter: char, frontmost_pid: Option<i32>) -> Option<&App> {
    let letter = letter.to_lowercase().next()?;
    let mut matches: Vec<&App> = apps
        .iter()
        .filter(|app| first_letter(&app.name) == Some(letter))
        .collect();
    matches.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.pid.cmp(&b.pid))
    });

    let current = matches
        .iter()
        .position(|app| Some(app.pid) == frontmost_pid);
    let next = current.map_or(0, |i| (i + 1) % matches.len());
    matches.get(next).copied()
}

fn first_letter(name: &str) -> Option<char> {
    name.chars().next()?.to_lowercase().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(pid: i32, name: &str) -> App {
        App {
            pid,
            name: name.to_string(),
        }
    }

    #[test]
    fn picks_the_app_that_starts_with_the_letter() {
        let apps = [app(1, "Safari"), app(2, "Mail")];
        assert_eq!(pick(&apps, 'm', None), Some(&apps[1]));
    }

    #[test]
    fn ignores_case() {
        let apps = [app(1, "slack")];
        assert_eq!(pick(&apps, 'S', None), Some(&apps[0]));
    }

    #[test]
    fn returns_none_when_no_app_matches() {
        let apps = [app(1, "Safari")];
        assert_eq!(pick(&apps, 'x', None), None);
    }

    #[test]
    fn cycles_to_the_next_match_when_one_is_in_front() {
        let apps = [app(1, "Safari"), app(2, "Slack"), app(3, "Spotify")];
        assert_eq!(pick(&apps, 's', Some(1)).unwrap().name, "Slack");
        assert_eq!(pick(&apps, 's', Some(2)).unwrap().name, "Spotify");
        assert_eq!(pick(&apps, 's', Some(3)).unwrap().name, "Safari");
    }

    #[test]
    fn starts_at_the_first_match_when_another_app_is_in_front() {
        let apps = [app(1, "Spotify"), app(2, "Safari"), app(3, "Mail")];
        assert_eq!(pick(&apps, 's', Some(3)).unwrap().name, "Safari");
    }
}
