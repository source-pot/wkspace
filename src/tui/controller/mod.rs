pub mod data;

pub fn run() -> anyhow::Result<()> {
    println!("wkspace controller (stub) — TUI to be implemented");
    // Wait for input so the pane doesn't immediately close in tmux.
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}
