use std::path::Path;
use wkspace::tui::tmux;

#[test]
fn slug_lowercases_and_replaces_non_alnum() {
    assert_eq!(tmux::repo_slug(Path::new("/Users/a/My Repo")), "my-repo");
}

#[test]
fn slug_collapses_repeated_dashes() {
    assert_eq!(
        tmux::repo_slug(Path::new("/x/foo___bar.git")),
        "foo-bar-git"
    );
}

#[test]
fn slug_trims_leading_and_trailing_dashes() {
    assert_eq!(tmux::repo_slug(Path::new("/x/--weird--")), "weird");
}

#[test]
fn slug_falls_back_when_empty() {
    assert_eq!(tmux::repo_slug(Path::new("/")), "repo");
}

#[test]
fn parses_modern_version() {
    assert_eq!(tmux::parse_version_output("tmux 3.4\n"), Some((3, 4)));
}

#[test]
fn parses_legacy_version() {
    assert_eq!(tmux::parse_version_output("tmux 2.8\n"), Some((2, 8)));
}

#[test]
fn parses_next_version() {
    assert_eq!(tmux::parse_version_output("tmux next-3.5\n"), Some((3, 5)));
}

#[test]
fn ignores_unparseable() {
    assert_eq!(tmux::parse_version_output("garbage"), None);
}

#[test]
fn version_at_least_compares_correctly() {
    assert!(tmux::version_at_least((3, 0), (3, 0)));
    assert!(tmux::version_at_least((3, 4), (3, 0)));
    assert!(tmux::version_at_least((4, 0), (3, 9)));
    assert!(!tmux::version_at_least((2, 9), (3, 0)));
}
