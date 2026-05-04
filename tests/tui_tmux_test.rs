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
