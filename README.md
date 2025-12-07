# SOZIN - Professional Network Interface Manager

[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

```
     ███████╗ ██████╗ ███████╗██╗███╗   ██╗
     ██╔════╝██╔═══██╗╚══███╔╝██║████╗  ██║
     ███████╗██║   ██║  ███╔╝ ██║██╔██╗ ██║
     ╚════██║██║   ██║ ███╔╝  ██║██║╚██╗██║
     ███████║╚██████╔╝███████╗██║██║ ╚████║
     ╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═╝  ╚═══╝
```

**⚠️ WARNING: This tool requires root privileges and is for authorized security testing only.**

SOZIN is a professional network interface management tool built with Rust for maximum performance. It provides WiFi scanning, monitor mode management, MAC spoofing, and network discovery capabilities through both an interactive TUI and CLI.

## 🎯 Features

### Core Capabilities
- **Interactive TUI**: Beautiful terminal user interface with real-time updates
- **Monitor Mode Management**: Enable/disable monitor mode on wireless interfaces
- **WiFi Scanning**: Discover nearby networks with signal strength, security info
- **Interface Control**: Bring interfaces up/down, rename, manage state
- **MAC Spoofing**: Change MAC addresses with random generation support

### Technical Features
- **Async Architecture**: Built on Tokio for high-performance async operations
- **Modular Design**: Clean separation of concerns (network, scanner, ui, banner)
- **Multiple Output Formats**: JSON support for scripting and automation
- **Cross-platform**: Works on any Linux system with standard networking tools

## 🚀 Installation

### Prerequisites
- Rust 1.70 or higher
- Linux system with `iw`, `ip` commands
- Root privileges for network operations

### Build from Source
```bash
# Clone the repository
git clone https://github.com/therealsylva/sozin.git
cd sozin

# Build the project
cargo build --release

# Install system-wide (optional)
sudo cp target/release/sozin /usr/local/bin/
```

## 📖 Usage

### Interactive TUI Mode (Default)
```bash
sudo sozin
# or
sudo sozin tui
```

### CLI Commands

#### List Interfaces
```bash
# List all interfaces
sudo sozin list

# List only wireless interfaces
sudo sozin list --wireless

# Output as JSON
sudo sozin list --json
```

#### Monitor Mode
```bash
# Enable monitor mode
sudo sozin monitor -i wlan0

# Disable monitor mode
sudo sozin monitor -i wlan0 --disable
```

#### WiFi Scanning
```bash
# Scan for networks
sudo sozin scan -i wlan0

# Output as JSON
sudo sozin scan -i wlan0 --json
```

#### Interface Control
```bash
# Bring interface up
sudo sozin up wlan0

# Bring interface down
sudo sozin down wlan0
```

#### MAC Spoofing
```bash
# Random MAC address
sudo sozin mac -i wlan0

# Specific MAC address
sudo sozin mac -i wlan0 -a 00:11:22:33:44:55
```

#### NetworkManager
```bash
# Restart NetworkManager
sudo sozin restart
```

## ⌨️ TUI Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch tabs |
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `q` | Quit |
| `?` | Toggle help |

### Interface Actions
| Key | Action |
|-----|--------|
| `m` | Toggle monitor mode |
| `u` | Bring interface up |
| `d` | Bring interface down |
| `M` | Spoof MAC address (random) |
| `r` | Refresh interfaces |

### Scanning
| Key | Action |
|-----|--------|
| `s` | Scan for WiFi networks |

### System
| Key | Action |
|-----|--------|
| `n` | Restart NetworkManager |

## 📁 Project Structure

```
sozin/
├── src/
│   ├── main.rs          # CLI interface and command handling
│   ├── network.rs       # Network interface management
│   ├── scanner.rs       # WiFi scanning and discovery
│   ├── ui.rs            # TUI implementation (ratatui)
│   └── banner.rs        # ASCII banner display
├── Cargo.toml           # Rust dependencies and metadata
└── README.md            # This file
```

## 🔧 Technical Details

### Dependencies
- **tokio**: Async runtime for high-performance operations
- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
- **clap**: Command-line argument parsing
- **serde**: Serialization for JSON output
- **nix**: Unix system calls for network operations

### Architecture
- **Async/Await**: All network operations are async for non-blocking execution
- **Modular Design**: Each component (network, scanner, ui) is independent
- **Error Handling**: Comprehensive error handling with anyhow/thiserror

## 🛡️ Security Considerations

### Ethical Use
- **Authorization Required**: Only use on networks you own or have permission to test
- **Root Privileges**: Required for network interface manipulation
- **Legal Compliance**: Ensure compliance with local laws and regulations

### Best Practices
- Use in isolated environments for testing
- Monitor and log all testing activities
- Respect network policies and regulations

## 📊 Output Formats

### JSON Interface List
```json
[
  {
    "name": "wlan0",
    "mac_address": "00:11:22:33:44:55",
    "ip_address": "192.168.1.100",
    "state": "Up",
    "interface_type": "Wireless",
    "driver": "iwlwifi"
  }
]
```

### JSON Network Scan
```json
[
  {
    "ssid": "MyNetwork",
    "bssid": "AA:BB:CC:DD:EE:FF",
    "channel": 6,
    "frequency": 2437,
    "signal_strength": -45,
    "security": "WPA2",
    "mode": "Infrastructure",
    "last_seen": "2025-12-07T01:00:00Z"
  }
]
```

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This tool is provided for educational and authorized security testing purposes only. The authors are not responsible for any misuse or damage caused by this tool. Users are solely responsible for ensuring they have proper authorization before testing any systems.

## 📞 Support

For issues, questions, or contributions:
- Create an issue on GitHub
- Follow responsible disclosure for security issues

---

**Built with ❤️ using Rust for maximum performance and reliability.**
