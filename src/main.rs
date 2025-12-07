mod banner;
mod network;
mod scanner;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "sozin")]
#[command(author = "therealsylva")]
#[command(version = "2.0.0")]
#[command(about = "Professional Network Interface Manager - WiFi scanning, monitor mode, and network discovery")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI mode
    Tui,
    
    /// List all network interfaces
    List {
        /// Show only wireless interfaces
        #[arg(short, long)]
        wireless: bool,
        
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Enable monitor mode on interface
    Monitor {
        /// Interface name
        #[arg(short, long)]
        interface: String,
        
        /// Disable monitor mode (set to managed)
        #[arg(short, long)]
        disable: bool,
    },
    
    /// Scan for WiFi networks
    Scan {
        /// Interface to scan with
        #[arg(short, long)]
        interface: String,
        
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Bring interface up
    Up {
        /// Interface name
        interface: String,
    },
    
    /// Bring interface down
    Down {
        /// Interface name
        interface: String,
    },
    
    /// Spoof MAC address
    Mac {
        /// Interface name
        #[arg(short, long)]
        interface: String,
        
        /// New MAC address (random if not specified)
        #[arg(short, long)]
        address: Option<String>,
    },
    
    /// Restart NetworkManager
    Restart,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check for root privileges
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("{}", "⚠ Warning: Some operations require root privileges".yellow());
    }

    match cli.command {
        Some(Commands::Tui) | None => {
            // Default to TUI mode
            banner::print_banner();
            ui::run_tui().await?;
        }
        
        Some(Commands::List { wireless, json }) => {
            let interfaces = if wireless {
                network::NetworkManager::get_wireless_interfaces()?
            } else {
                network::NetworkManager::get_interfaces()?
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&interfaces)?);
            } else {
                banner::print_mini_banner();
                println!();
                for iface in &interfaces {
                    let state_color = match iface.state {
                        network::InterfaceState::Up => "green",
                        network::InterfaceState::Down => "red",
                        network::InterfaceState::Unknown => "yellow",
                    };
                    
                    println!(
                        "  {} {} [{}] - {} {}",
                        match iface.interface_type {
                            network::InterfaceType::Wireless => "📶",
                            network::InterfaceType::Ethernet => "🔌",
                            network::InterfaceType::Loopback => "🔄",
                            network::InterfaceType::Virtual => "🌐",
                            network::InterfaceType::Unknown => "❓",
                        },
                        iface.name.bold(),
                        iface.state.to_string().color(state_color),
                        iface.interface_type,
                        iface.mac_address.as_deref().unwrap_or("").bright_black()
                    );
                }
                println!();
                println!("  {} interfaces found", interfaces.len().to_string().cyan());
            }
        }
        
        Some(Commands::Monitor { interface, disable }) => {
            banner::print_mini_banner();
            
            if disable {
                println!("  {} Disabling monitor mode on {}...", "»".cyan(), interface.bold());
                network::NetworkManager::disable_monitor_mode(&interface).await?;
                println!("  {} Monitor mode disabled", "✓".green());
            } else {
                println!("  {} Enabling monitor mode on {}...", "»".cyan(), interface.bold());
                network::NetworkManager::enable_monitor_mode(&interface).await?;
                println!("  {} Monitor mode enabled", "✓".green());
            }
        }
        
        Some(Commands::Scan { interface, json }) => {
            if !json {
                banner::print_mini_banner();
                println!();
                println!("  {} Scanning on {}...", "»".cyan(), interface.bold());
            }
            
            let mut wifi_scanner = scanner::WifiScanner::new(&interface);
            let networks = wifi_scanner.scan().await?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&networks)?);
            } else {
                println!("  {} Found {} networks\n", "✓".green(), networks.len().to_string().cyan());
                
                println!(
                    "  {:<25} {:<18} {:>4} {:>8} {}",
                    "SSID".cyan(),
                    "BSSID".cyan(),
                    "CH".cyan(),
                    "Signal".cyan(),
                    "Security".cyan()
                );
                println!("  {}", "─".repeat(70).bright_black());
                
                for net in &networks {
                    let signal_color = if net.signal_strength > -50 {
                        "green"
                    } else if net.signal_strength > -70 {
                        "yellow"
                    } else {
                        "red"
                    };
                    
                    println!(
                        "  {:<25} {:<18} {:>4} {:>8} {}",
                        if net.ssid.len() > 24 {
                            format!("{}...", &net.ssid[..21])
                        } else {
                            net.ssid.clone()
                        },
                        net.bssid,
                        net.channel,
                        format!("{}dBm", net.signal_strength).color(signal_color),
                        net.security
                    );
                }
            }
        }
        
        Some(Commands::Up { interface }) => {
            banner::print_mini_banner();
            println!("  {} Bringing up {}...", "»".cyan(), interface.bold());
            network::NetworkManager::bring_up(&interface).await?;
            println!("  {} {} is now UP", "✓".green(), interface);
        }
        
        Some(Commands::Down { interface }) => {
            banner::print_mini_banner();
            println!("  {} Bringing down {}...", "»".cyan(), interface.bold());
            network::NetworkManager::bring_down(&interface).await?;
            println!("  {} {} is now DOWN", "✓".green(), interface);
        }
        
        Some(Commands::Mac { interface, address }) => {
            banner::print_mini_banner();
            let new_mac = address.unwrap_or_else(|| network::NetworkManager::generate_random_mac());
            println!("  {} Changing MAC on {} to {}...", "»".cyan(), interface.bold(), new_mac.yellow());
            network::NetworkManager::spoof_mac(&interface, &new_mac).await?;
            println!("  {} MAC address changed to {}", "✓".green(), new_mac.green());
        }
        
        Some(Commands::Restart) => {
            banner::print_mini_banner();
            println!("  {} Restarting NetworkManager...", "»".cyan());
            network::NetworkManager::restart_network_manager().await?;
            println!("  {} NetworkManager restarted", "✓".green());
        }
    }

    Ok(())
}
