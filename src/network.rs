use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
    pub state: InterfaceState,
    pub interface_type: InterfaceType,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceState {
    Up,
    Down,
    Unknown,
}

impl std::fmt::Display for InterfaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceState::Up => write!(f, "UP"),
            InterfaceState::Down => write!(f, "DOWN"),
            InterfaceState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    Wireless,
    Ethernet,
    Loopback,
    Virtual,
    Unknown,
}

impl std::fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterfaceType::Wireless => write!(f, "Wireless"),
            InterfaceType::Ethernet => write!(f, "Ethernet"),
            InterfaceType::Loopback => write!(f, "Loopback"),
            InterfaceType::Virtual => write!(f, "Virtual"),
            InterfaceType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirelessMode {
    Managed,
    Monitor,
    Master,
    Adhoc,
    Unknown,
}

impl std::fmt::Display for WirelessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WirelessMode::Managed => write!(f, "Managed"),
            WirelessMode::Monitor => write!(f, "Monitor"),
            WirelessMode::Master => write!(f, "Master"),
            WirelessMode::Adhoc => write!(f, "Ad-Hoc"),
            WirelessMode::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Network manager for interface operations
pub struct NetworkManager;

impl NetworkManager {
    /// Get all network interfaces
    pub fn get_interfaces() -> Result<Vec<NetworkInterface>> {
        let output = Command::new("ip")
            .args(["-o", "link", "show"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get network interfaces"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in stdout.lines() {
            if let Some(iface) = Self::parse_interface_line(line) {
                interfaces.push(iface);
            }
        }

        Ok(interfaces)
    }

    /// Get wireless interfaces only
    pub fn get_wireless_interfaces() -> Result<Vec<NetworkInterface>> {
        let interfaces = Self::get_interfaces()?;
        Ok(interfaces
            .into_iter()
            .filter(|i| i.interface_type == InterfaceType::Wireless)
            .collect())
    }

    fn parse_interface_line(line: &str) -> Option<NetworkInterface> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        // Extract interface name (remove trailing colon)
        let name = parts[1].trim_end_matches(':').to_string();
        
        // Skip loopback for most operations
        if name == "lo" {
            return Some(NetworkInterface {
                name,
                mac_address: None,
                ip_address: None,
                state: InterfaceState::Up,
                interface_type: InterfaceType::Loopback,
                driver: None,
            });
        }

        // Determine state
        let state = if line.contains("state UP") {
            InterfaceState::Up
        } else if line.contains("state DOWN") {
            InterfaceState::Down
        } else {
            InterfaceState::Unknown
        };

        // Determine interface type
        let interface_type = Self::detect_interface_type(&name);

        // Extract MAC address
        let mac_address = Self::get_mac_address(&name);

        // Extract IP address
        let ip_address = Self::get_ip_address(&name);

        // Get driver info
        let driver = Self::get_driver(&name);

        Some(NetworkInterface {
            name,
            mac_address,
            ip_address,
            state,
            interface_type,
            driver,
        })
    }

    fn detect_interface_type(name: &str) -> InterfaceType {
        // Check if wireless by looking at /sys/class/net/<iface>/wireless
        let wireless_path = format!("/sys/class/net/{}/wireless", name);
        if std::path::Path::new(&wireless_path).exists() {
            return InterfaceType::Wireless;
        }

        // Check by name patterns
        if name.starts_with("wl") || name.starts_with("wlan") || name.starts_with("wifi") {
            return InterfaceType::Wireless;
        }

        if name.starts_with("eth") || name.starts_with("en") {
            return InterfaceType::Ethernet;
        }

        if name.starts_with("veth") || name.starts_with("docker") || name.starts_with("br-") {
            return InterfaceType::Virtual;
        }

        InterfaceType::Unknown
    }

    fn get_mac_address(name: &str) -> Option<String> {
        let path = format!("/sys/class/net/{}/address", name);
        std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    fn get_ip_address(name: &str) -> Option<String> {
        let output = Command::new("ip")
            .args(["-4", "addr", "show", name])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("inet ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].split('/').next()?.to_string());
                }
            }
        }
        None
    }

    fn get_driver(name: &str) -> Option<String> {
        let path = format!("/sys/class/net/{}/device/driver", name);
        std::fs::read_link(&path)
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
    }

    /// Get current wireless mode
    pub fn get_wireless_mode(interface: &str) -> Result<WirelessMode> {
        let output = Command::new("iw")
            .args(["dev", interface, "info"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.contains("type") {
                if line.contains("monitor") {
                    return Ok(WirelessMode::Monitor);
                } else if line.contains("managed") {
                    return Ok(WirelessMode::Managed);
                } else if line.contains("AP") || line.contains("master") {
                    return Ok(WirelessMode::Master);
                } else if line.contains("IBSS") || line.contains("ad-hoc") {
                    return Ok(WirelessMode::Adhoc);
                }
            }
        }

        Ok(WirelessMode::Unknown)
    }

    /// Enable monitor mode on interface
    pub async fn enable_monitor_mode(interface: &str) -> Result<()> {
        // Bring interface down
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .await?;

        // Set monitor mode
        let output = AsyncCommand::new("iw")
            .args(["dev", interface, "set", "type", "monitor"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to set monitor mode: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Bring interface up
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "up"])
            .output()
            .await?;

        Ok(())
    }

    /// Disable monitor mode (set to managed)
    pub async fn disable_monitor_mode(interface: &str) -> Result<()> {
        // Bring interface down
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .await?;

        // Set managed mode
        let output = AsyncCommand::new("iw")
            .args(["dev", interface, "set", "type", "managed"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to set managed mode: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Bring interface up
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "up"])
            .output()
            .await?;

        Ok(())
    }

    /// Bring interface up
    pub async fn bring_up(interface: &str) -> Result<()> {
        let output = AsyncCommand::new("ip")
            .args(["link", "set", interface, "up"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to bring up interface: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Bring interface down
    pub async fn bring_down(interface: &str) -> Result<()> {
        let output = AsyncCommand::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to bring down interface: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Rename interface
    pub async fn rename_interface(interface: &str, new_name: &str) -> Result<()> {
        // Bring interface down first
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .await?;

        // Rename
        let output = AsyncCommand::new("ip")
            .args(["link", "set", interface, "name", new_name])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to rename interface: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Bring interface up with new name
        AsyncCommand::new("ip")
            .args(["link", "set", new_name, "up"])
            .output()
            .await?;

        Ok(())
    }

    /// Restart NetworkManager
    pub async fn restart_network_manager() -> Result<()> {
        let output = AsyncCommand::new("systemctl")
            .args(["restart", "NetworkManager"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to restart NetworkManager: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Spoof MAC address
    pub async fn spoof_mac(interface: &str, new_mac: &str) -> Result<()> {
        // Bring interface down
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .await?;

        // Change MAC
        let output = AsyncCommand::new("ip")
            .args(["link", "set", interface, "address", new_mac])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to change MAC address: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Bring interface up
        AsyncCommand::new("ip")
            .args(["link", "set", interface, "up"])
            .output()
            .await?;

        Ok(())
    }

    /// Generate random MAC address
    pub fn generate_random_mac() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // First byte should have bit 1 clear (unicast) and bit 0 set (locally administered)
        let first_byte = (rng.gen::<u8>() & 0xFC) | 0x02;
        
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            first_byte,
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>()
        )
    }

    /// Set wireless channel
    pub async fn set_channel(interface: &str, channel: u32) -> Result<()> {
        let output = AsyncCommand::new("iw")
            .args(["dev", interface, "set", "channel", &channel.to_string()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to set channel: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
