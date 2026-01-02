use colored::*;

pub fn print_banner() {
    let banner = format!(
        r#"
    {}
     ███████╗ ██████╗ ███████╗██╗███╗   ██╗
     ██╔════╝██╔═══██╗╚══███╔╝██║████╗  ██║
     ███████╗██║   ██║  ███╔╝ ██║██╔██╗ ██║
     ╚════██║██║   ██║ ███╔╝  ██║██║╚██╗██║
     ███████║╚██████╔╝███████╗██║██║ ╚████║
     ╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═╝  ╚═══╝
    {}
    
    {} Professional Network Interface Manager
    {} WiFi Scanning & Network Discovery
    {} Version 2.1.0
    {} by therealsylva
    
    {} Requires root privileges for network operations
    {}
"#,
        "═".repeat(50).bright_black(),
        "═".repeat(50).bright_black(),
        "»".bright_cyan(),
        "»".bright_cyan(),
        "»".bright_cyan(),
        "»".bright_cyan(),
        "⚠".yellow().bold(),
        "═".repeat(50).bright_black(),
    );
    println!("{}", banner);
}

pub fn print_mini_banner() {
    println!(
        "{}",
        "  SOZIN v2.1.0 | Network Interface Manager".bright_cyan()
    );
    println!("{}", "═".repeat(50).bright_black());
}
