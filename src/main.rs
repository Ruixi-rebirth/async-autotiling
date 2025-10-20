use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use swayipc_async::{Connection, Event, EventType, Node, NodeLayout, NodeType, WindowChange};

/// Automatically switch between horizontal/vertical split layout for sway/i3
#[derive(Parser, Debug)]
#[command(
    name = "async-autotiling",
    version,
    author = "Ruixi-rebirth",
    about = "Automatically switch between horizontal/vertical split layout for sway/i3.",
    long_about = "A small and efficient tool that runs in the background to listen for window events in sway/i3.
It intelligently predicts the best split direction (horizontal or vertical) for the next window
based on the dimensions of the currently focused window, automating the manual `mod+h`/`mod+v`
operations for a smoother tiling experience."
)]
struct Args {
    /// Sets the aspect ratio threshold to trigger a vertical split.
    /// When `window_height > window_width / ratio`, the next split will be vertical.
    /// A value of 1.0 means any window taller than it is wide will trigger a vertical split.
    /// 1.618 (golden ratio) is a popular alternative.
    #[arg(long, default_value_t = 1.0, value_name = "RATIO")]
    ratio: f64,

    /// Restricts the script to run only on one or more specified workspaces.
    /// Provide a comma-separated list. If empty, the script will run on all workspaces.
    /// Example: --workspace 1,dev,"Web Browsing"
    #[arg(
        long,
        value_delimiter = ',',
        use_value_delimiter = true,
        value_name = "NAMES"
    )]
    workspace: Vec<String>,

    /// Run the logic once and exit immediately.
    /// Useful for scripting or one-off tests.
    #[arg(long, default_value_t = false)]
    once: bool,

    /// Quiet mode, suppresses all log output.
    /// Ideal for running as a silent background service.
    #[arg(long, short, default_value_t = false)]
    quiet: bool,
}

/// A simple logger that respects the --quiet flag.
struct Logger {
    is_quiet: bool,
}

impl Logger {
    fn new(is_quiet: bool) -> Self {
        Self { is_quiet }
    }
    fn info(&self, msg: &str) {
        if !self.is_quiet {
            println!("{}", msg);
        }
    }
    fn error(&self, msg: &str) {
        if !self.is_quiet {
            eprintln!("{}", msg);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let logger = Logger::new(args.quiet);

    let mut cmd_conn = Connection::new().await?;

    if args.once {
        run_autotile(&mut cmd_conn, &args, &logger).await?;
        return Ok(());
    }

    logger.info("AutoTiling-rs started — listening for window events...");

    let mut event_conn = Connection::new().await?;
    let mut events = event_conn.subscribe([EventType::Window]).await?;

    loop {
        match events.next().await {
            Some(Ok(Event::Window(ev))) => {
                if matches!(ev.change, WindowChange::Focus) {
                    if let Err(e) = run_autotile(&mut cmd_conn, &args, &logger).await {
                        logger.error(&format!("Error during auto-tiling: {}", e));
                    }
                }
            }
            Some(Err(e)) => {
                logger.error(&format!(
                    "Event stream error: {}. Attempting to reconnect...",
                    e
                ));
                loop {
                    if let Ok(new_conn) = Connection::new().await {
                        event_conn = new_conn;
                        if let Ok(new_events) = event_conn.subscribe([EventType::Window]).await {
                            events = new_events;
                            logger.info("Reconnected event stream successfully.");
                            break;
                        }
                    }
                    logger.error("Failed to reconnect. Retrying in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
            None => break,
            _ => {}
        }
    }

    Ok(())
}

/// Core logic: determine and switch the layout.
async fn run_autotile(conn: &mut Connection, args: &Args, logger: &Logger) -> Result<()> {
    let tree = conn.get_tree().await?;

    if !args.workspace.is_empty() {
        if let Some(ws_name) = get_focused_workspace_name(conn).await? {
            if !args.workspace.contains(&ws_name) {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    // Find the parent of the focused node. This is more direct and efficient.
    // This closure means "find the node whose children contain the focused node".
    if let Some(parent) = tree.find_focused_as_ref(|n| n.nodes.iter().any(|child| child.focused)) {
        // Now, find the actual focused node within that parent.
        if let Some(focused_node) = parent.nodes.iter().find(|n| n.focused) {
            if should_skip(focused_node) {
                return Ok(());
            }

            let rect = focused_node.rect;
            let height = rect.height as f64;
            let width = rect.width as f64;

            let new_layout = if height > width / args.ratio {
                NodeLayout::SplitV
            } else {
                NodeLayout::SplitH
            };

            // If the parent's layout is already what we want, do nothing.
            if new_layout != parent.layout {
                let cmd = if new_layout == NodeLayout::SplitV {
                    "splitv"
                } else {
                    "splith"
                };
                conn.run_command(cmd).await?;
                logger.info(&format!(
                    "Focus changed -> Next split direction set to '{}'",
                    cmd
                ));
            }
        }
    }
    Ok(())
}

/// Check if a node should be skipped (using more robust checks).
fn should_skip(node: &Node) -> bool {
    // A more reliable way to detect fullscreen is checking the `percent`.
    let is_fullscreen = node.percent.map_or(false, |p| p > 1.0);
    let is_floating = matches!(node.node_type, NodeType::FloatingCon);
    matches!(node.layout, NodeLayout::Tabbed | NodeLayout::Stacked) || is_fullscreen || is_floating
}

/// Get the name of the currently focused workspace.
async fn get_focused_workspace_name(conn: &mut Connection) -> Result<Option<String>> {
    let workspaces = conn.get_workspaces().await?;
    Ok(workspaces
        .into_iter()
        .find(|ws| ws.focused)
        .map(|ws| ws.name))
}
