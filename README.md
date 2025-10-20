# async-autotiling

async-autotiling is a small and efficient Rust utility for sway/i3 that automatically switches the split layout (horizontal or vertical) based on the dimensions of the currently focused window. It listens for window events and sets the next split direction to provide a smoother tiling experience.

## Features

- Listens for window events (focus) and adjusts the next split direction.
- Uses a configurable aspect ratio threshold to decide between vertical and horizontal splits.
- Optionally restrict behavior to specified workspaces.
- Option to run once (for scripting or testing) or run continuously as a background service.
- Quiet mode to suppress log output.

## Installation

There are several ways to install async-autotiling:

### Using Cargo (Traditional Method)

1. Ensure you have the Rust toolchain installed (rustup + cargo).
2. Clone the repository:

```bash
git clone https://github.com/Ruixi-rebirth/async-autotiling.git

cd async-autotiling
```

3. Build the project:

```bash
cargo build --release
```

4. Copy or symlink the resulting binary (target/release/async-autotiling) to a location in your PATH.

### Using Nix Flakes

#### Build via Flakes

1. Ensure you have Nix installed with flake support (experimental mode enabled).
2. Clone the repository:

```bash
git clone https://github.com/Ruixi-rebirth/async-autotiling.git

cd async-autotiling
```

3. Build the project using Nix:

```bash
nix build

```

This builds the project as defined in flake.nix, with the output symlinked as `./result`.

#### Install with nix profile

Install async-autotiling directly to your user profile:

```bash
nix profile install github:Ruixi-rebirth/async-autotiling

```

This command adds async-autotiling to your Nix user profile.

#### NixOS Configuration

For NixOS users, you can add async-autotiling to your system packages. For example, add the following to your configuration.nix:

```nix
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    (import (builtins.getFlake "github:Ruixi-rebirth/async-autotiling") { })
    .packages.${system}.async-autotiling
  ];
}
```

Then, rebuild your system configuration with:

```bash
sudo nixos-rebuild switch
```

Note: This tool requires sway or i3 with the IPC socket accessible from the user running the program.

## Usage

Run the binary with `--help` to see available options:

> async-autotiling --help
>
> Options
>
> - --ratio RATIO
>   Sets the aspect ratio threshold to trigger a vertical split. If `window_height > window_width / ratio` then it chooses a vertical split. Default: 1.0
> - --workspace NAMES
>   Restrict the behavior to specific workspace names (comma-separated). Example: `--workspace 1,dev,"Web Browsing"`
> - --once
>   Run the logic once and exit. Useful for scripting or one-off checks.
> - -q, --quiet
>   Suppress log output.
>
> Example:
>
> async-autotiling --ratio 1.618<br>
> async-autotiling --workspace dev,1 --quiet<br>
> async-autotiling --once

## How it works

1. Connects to sway/i3 IPC and listens for window events.
2. When a window event occurs (focus), it finds the currently focused window.
3. If the window is not floating, tabbed, stacked, or fullscreen and the workspace restriction (if any) matches, it compares the window's width and height against the configured ratio.
4. It sends the `splitv` or `splith` command to sway/i3 to set the layout for the next split.

## Configuration

There is no external configuration file. Use command-line flags to adjust behavior.

## Contributing

Contributions are welcome. Please open an issue or a pull request.

## Acknowledgments

Built using:

- swayipc-async
- tokio
- clap
- anyhow
