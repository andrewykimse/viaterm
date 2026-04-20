# viaterm

A terminal-based [VIA](https://usevia.app) keyboard configurator built with Rust and [Ratatui](https://ratatui.rs). Configure your QMK keyboard keymaps directly from the terminal — no browser required.

## Features

- Auto-detects VIA-compatible keyboards over USB
- Fetches keyboard definitions automatically from usevia.app (cached locally)
- 2D keyboard layout rendering with box-drawing characters
- Keymap editing with categorized keycode picker and search
- Multi-layer support
- Writes changes directly to the keyboard in real-time

## Installation

### Cargo

```sh
cargo install --git https://github.com/andrewkim/viaterm
```

Or build from source:

```sh
git clone https://github.com/andrewkim/viaterm
cd viaterm
cargo install --path .
```

### Nix

Run without installing:

```sh
nix run github:andrewkim/viaterm
```

Or add to your flake inputs:

```nix
{
  inputs.viaterm.url = "github:andrewkim/viaterm";

  outputs = { self, nixpkgs, viaterm, ... }: {
    # Add to your packages
    environment.systemPackages = [ viaterm.packages.${system}.default ];
  };
}
```

Build with Nix directly:

```sh
nix build github:andrewkim/viaterm
./result/bin/viaterm
```

### Linux: USB permissions

On Linux you need a udev rule so your user can access HID devices. Create `/etc/udev/rules.d/99-viaterm.rules`:

```
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", MODE="0660", TAG+="uaccess"
```

Then reload:

```sh
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Usage

```sh
viaterm
```

That's it. Plug in your VIA-compatible keyboard, run `viaterm`, and it will scan for devices automatically. Select your keyboard and the definition is fetched from usevia.app.

To use a local definition file instead:

```sh
viaterm --definition path/to/keyboard.json
# or
viaterm -d path/to/keyboard.json
```

## Keybindings

### Device selection

| Key       | Action   |
|-----------|----------|
| `↑` `↓`  | Navigate |
| `Enter`   | Connect  |
| `r`       | Rescan   |
| `q`       | Quit     |

### Keymap editor

| Key              | Action              |
|------------------|----------------------|
| `←` `↑` `↓` `→` | Move selection       |
| `Enter`          | Open keycode picker  |
| `Tab` / `S-Tab`  | Next / prev layer    |
| `w`              | Save changes to keyboard |
| `d`              | Disconnect           |
| `q`              | Quit                 |

### Keycode picker

| Key       | Action            |
|-----------|-------------------|
| `↑` `↓`  | Select keycode    |
| `←` `→`  | Switch category   |
| `Enter`   | Confirm           |
| `Esc`     | Cancel            |
| Type      | Search keycodes   |

## Requirements

- A keyboard with [VIA-compatible](https://caniusevia.com) QMK firmware
- USB connection (Bluetooth is not supported)
- macOS, Linux, or Windows

## License

GPL-3.0
