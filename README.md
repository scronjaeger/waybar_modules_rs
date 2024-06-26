# Waybar Modules in Rust

This repository contains custom modules written in Rust for [Waybar](https://github.com/Alexays/Waybar). Waybar is a highly customizable bar for Wayland, primarily for sway and wlr-workspaces. These modules provide additional functionality such as disk usage, network activity, and CPU usage monitoring.

## Repository Structure

The repository is structured into different modules, each providing specific functionality for Waybar:

- `disk_module`: Monitors disk read and write activity.
- `inet_module`: Monitors network activity and IP address information.
- `cpu_module`: Monitors CPU usage.
- `bar_module`: A shared library that provides utility functions used by the other modules.

## Modules

### Disk Module

The `disk_module` monitors disk read and write activity and displays it in Waybar. It reads from the `/sys/class/block` directory and outputs the data in an easily readable format with color coding.

**Main file:** `disk_module/src/main.rs`

**Cargo.toml:** `disk_module/Cargo.toml`

**Dependencies:** `bar_module`

### Inet Module

The `inet_module` monitors network activity, including sent and received bytes for network interfaces, as well as detailed IP address information. It utilizes the `reqwest` library to fetch external IP address and `if-addrs` to fetch local IP addresses.

**Main file:** `inet_module/src/main.rs`

**Cargo.toml:** `inet_module/Cargo.toml`

**Dependencies:** `bar_module`, `reqwest`, `if-addrs`

### CPU Module

The `cpu_module` monitors CPU usage, providing real-time data on CPU activity. It reads from `/proc/stat` to gather CPU statistics and outputs this data with color coding in Waybar.

**Main file:** `cpu_module/src/main.rs`

**Cargo.toml:** `cpu_module/Cargo.toml`

**Dependencies:** `bar_module`

### Bar Module

The `bar_module` is a shared library containing utility functions for normalizing, humanizing, and color coding values. It is used by the other modules to ensure consistent data formatting and color representation.

**Main file:** `bar_module/src/lib.rs`

**Cargo.toml:** `bar_module/Cargo.toml`

## Usage

### Build

To build and run any of the modules, navigate to the respective module’s directory and use Cargo:

```sh
cd <module_name>
cargo build --release
./target/release/<module_name>
```

For example, to build and run the `disk_module`:

```sh
cd disk_module
cargo build --release
./target/release/disk_module
```

### Configuration in Waybar

To add any of these modules to your Waybar configuration, first compile the module and then add an appropriate entry in your `~/.config/waybar/config` file.

For example, to add the `disk_module`:

```json
"custom/disk": {
  "exec": "/path/to/disk_module",
  "interval": "once",
  "escape": false,
  "return-type": "json"
},
"modules-left": ["custom/disk"]
```

Repeat the process for each module you want to include, adjusting the `exec` path to point to the compiled binary.

## License

This project is licensed under the GPL License. See the `LICENSE` file for details.
