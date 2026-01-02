use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command as AsyncCommand;
use tokio::time::{timeout, Duration};

/// WiFi network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub channel: u32,
    pub frequency: u32,
    pub signal_strength: i32,
    pub security: SecurityType,
    pub mode: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityType {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    WPA2Enterprise,
    Unknown,
}

impl std::fmt::Display for SecurityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityType::Open => write!(f, "Open"),
            SecurityType::WEP => write!(f, "WEP"),
            SecurityType::WPA => write!(f, "WPA"),
            SecurityType::WPA2 => write!(f, "WPA2"),
            SecurityType::WPA3 => write!(f, "WPA3"),
            SecurityType::WPA2Enterprise => write!(f, "WPA2-Enterprise"),
            SecurityType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// WiFi scanner for network discovery
pub struct WifiScanner {
    interface: String,
    networks: HashMap<String, WifiNetwork>,
}

impl WifiScanner {
    pub fn new(interface: &str) -> Self {
        Self {
            interface: interface.to_string(),
            networks: HashMap::new(),
        }
    }

    /// Scan for WiFi networks using iw
    pub async fn scan(&mut self) -> Result<Vec<WifiNetwork>> {
        // Trigger scan
        let scan_result = timeout(
            Duration::from_secs(10),
            AsyncCommand::new("iw")
                .args(["dev", &self.interface, "scan"])
                .output(),
        )
        .await??;

        if !scan_result.status.success() {
            // Try with sudo if permission denied
            let stderr = String::from_utf8_lossy(&scan_result.stderr);
            if stderr.contains("Operation not permitted") || stderr.contains("Network is down") {
                return Err(anyhow!(
                    "Scan failed: {}. Try running with sudo or ensure interface is up.",
                    stderr.trim()
                ));
            }
            return Err(anyhow!("Scan failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&scan_result.stdout);
        self.parse_scan_results(&stdout)
    }

    /// Parse iw scan output
    fn parse_scan_results(&mut self, output: &str) -> Result<Vec<WifiNetwork>> {
        let mut networks = Vec::new();
        let mut current_network: Option<WifiNetworkBuilder> = None;

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("BSS ") {
                // Save previous network if exists
                if let Some(builder) = current_network.take() {
                    if let Some(network) = builder.build() {
                        self.networks.insert(network.bssid.clone(), network.clone());
                        networks.push(network);
                    }
                }

                // Start new network
                let bssid = line
                    .strip_prefix("BSS ")
                    .and_then(|s| s.split('(').next())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                current_network = Some(WifiNetworkBuilder::new(bssid));
            } else if let Some(ref mut builder) = current_network {
                if line.starts_with("SSID:") {
                    builder.ssid = line.strip_prefix("SSID:").map(|s| s.trim().to_string());
                } else if line.starts_with("freq:") {
                    if let Some(freq_str) = line.strip_prefix("freq:") {
                        builder.frequency = freq_str.trim().parse().ok();
                        builder.channel = Self::freq_to_channel(builder.frequency.unwrap_or(0));
                    }
                } else if line.starts_with("signal:") {
                    if let Some(signal_str) = line.strip_prefix("signal:") {
                        let signal_str = signal_str.trim().split_whitespace().next().unwrap_or("0");
                        builder.signal_strength = signal_str.parse().ok();
                    }
                } else if line.contains("WPA") || line.contains("RSN") || line.contains("WEP") {
                    builder.update_security(line);
                } else if line.starts_with("DS Parameter set:") {
                    if let Some(ch_str) = line.split("channel").nth(1) {
                        builder.channel = ch_str.trim().parse().ok();
                    }
                }
            }
        }

        // Don't forget the last network
        if let Some(builder) = current_network {
            if let Some(network) = builder.build() {
                self.networks.insert(network.bssid.clone(), network.clone());
                networks.push(network);
            }
        }

        // Sort by signal strength (strongest first)
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));

        Ok(networks)
    }

    /// Convert frequency to channel number
    fn freq_to_channel(freq: u32) -> Option<u32> {
        match freq {
            2412 => Some(1),
            2417 => Some(2),
            2422 => Some(3),
            2427 => Some(4),
            2432 => Some(5),
            2437 => Some(6),
            2442 => Some(7),
            2447 => Some(8),
            2452 => Some(9),
            2457 => Some(10),
            2462 => Some(11),
            2467 => Some(12),
            2472 => Some(13),
            2484 => Some(14),
            // 5GHz channels
            5180 => Some(36),
            5200 => Some(40),
            5220 => Some(44),
            5240 => Some(48),
            5260 => Some(52),
            5280 => Some(56),
            5300 => Some(60),
            5320 => Some(64),
            5500 => Some(100),
            5520 => Some(104),
            5540 => Some(108),
            5560 => Some(112),
            5580 => Some(116),
            5600 => Some(120),
            5620 => Some(124),
            5640 => Some(128),
            5660 => Some(132),
            5680 => Some(136),
            5700 => Some(140),
            5720 => Some(144),
            5745 => Some(149),
            5765 => Some(153),
            5785 => Some(157),
            5805 => Some(161),
            5825 => Some(165),
            _ => None,
        }
    }

    #[allow(dead_code)]
    /// Get cached networks
    pub fn get_cached_networks(&self) -> Vec<WifiNetwork> {
        self.networks.values().cloned().collect()
    }

    #[allow(dead_code)]
    /// Clear cached networks
    pub fn clear_cache(&mut self) {
        self.networks.clear();
    }

    #[allow(dead_code)]
    /// Get network by BSSID
    pub fn get_network(&self, bssid: &str) -> Option<&WifiNetwork> {
        self.networks.get(bssid)
    }
}

/// Builder for WiFi network parsing
struct WifiNetworkBuilder {
    bssid: String,
    ssid: Option<String>,
    channel: Option<u32>,
    frequency: Option<u32>,
    signal_strength: Option<i32>,
    security: SecurityType,
}

impl WifiNetworkBuilder {
    fn new(bssid: String) -> Self {
        Self {
            bssid,
            ssid: None,
            channel: None,
            frequency: None,
            signal_strength: None,
            security: SecurityType::Open,
        }
    }

    fn update_security(&mut self, line: &str) {
        if line.contains("WPA3") {
            self.security = SecurityType::WPA3;
        } else if line.contains("RSN") || line.contains("WPA2") {
            if line.contains("802.1X") || line.contains("EAP") {
                self.security = SecurityType::WPA2Enterprise;
            } else if self.security != SecurityType::WPA3 {
                self.security = SecurityType::WPA2;
            }
        } else if line.contains("WPA") && self.security == SecurityType::Open {
            self.security = SecurityType::WPA;
        } else if line.contains("WEP") && self.security == SecurityType::Open {
            self.security = SecurityType::WEP;
        }
    }

    fn build(self) -> Option<WifiNetwork> {
        Some(WifiNetwork {
            ssid: self.ssid.unwrap_or_else(|| "<hidden>".to_string()),
            bssid: self.bssid,
            channel: self.channel.unwrap_or(0),
            frequency: self.frequency.unwrap_or(0),
            signal_strength: self.signal_strength.unwrap_or(-100),
            security: self.security,
            mode: "Infrastructure".to_string(),
            last_seen: chrono::Utc::now(),
        })
    }
}

#[allow(dead_code)]
/// Continuous scanner for real-time monitoring
pub struct ContinuousScanner {
    scanner: WifiScanner,
    scan_interval: Duration,
}

#[allow(dead_code)]
impl ContinuousScanner {
    pub fn new(interface: &str, scan_interval_secs: u64) -> Self {
        Self {
            scanner: WifiScanner::new(interface),
            scan_interval: Duration::from_secs(scan_interval_secs),
        }
    }

    /// Run continuous scanning
    pub async fn run<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(Vec<WifiNetwork>),
    {
        loop {
            match self.scanner.scan().await {
                Ok(networks) => callback(networks),
                Err(e) => eprintln!("Scan error: {}", e),
            }
            tokio::time::sleep(self.scan_interval).await;
        }
    }

    /// Get scanner reference
    pub fn scanner(&self) -> &WifiScanner {
        &self.scanner
    }

    /// Get mutable scanner reference
    pub fn scanner_mut(&mut self) -> &mut WifiScanner {
        &mut self.scanner
    }
}

/// Signal strength to quality percentage
pub fn signal_to_quality(signal_dbm: i32) -> u8 {
    if signal_dbm >= -50 {
        100
    } else if signal_dbm <= -100 {
        0
    } else {
        (2 * (signal_dbm + 100)) as u8
    }
}

/// Signal strength to bar representation
pub fn signal_to_bars(signal_dbm: i32) -> &'static str {
    let quality = signal_to_quality(signal_dbm);
    match quality {
        80..=100 => "████",
        60..=79 => "███░",
        40..=59 => "██░░",
        20..=39 => "█░░░",
        _ => "░░░░",
    }
}
