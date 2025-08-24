#!/bin/bash

# Function to check for root privileges
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo "Please run as root"
        exit
    fi
}

# Function to get network interfaces
get_interfaces() {
    ip -o link show | awk -F': ' '{print $2}'
}

# Function to enable monitor mode
enable_monitor_mode() {
    interface=$(get_interfaces | gum filter --placeholder "Select interface to enable monitor mode")
    if [ -n "$interface" ]; then
        gum spin --spinner dot --title "Enabling monitor mode on $interface..." -- sleep 2
        ip link set "$interface" down
        iw dev "$interface" set type monitor
        ip link set "$interface" up
        echo "Monitor mode enabled on $interface"
    fi
}

# Function to disable monitor mode
disable_monitor_mode() {
    interface=$(get_interfaces | gum filter --placeholder "Select interface to disable monitor mode")
    if [ -n "$interface" ]; then
        gum spin --spinner dot --title "Disabling monitor mode on $interface..." -- sleep 2
        ip link set "$interface" down
        iw dev "$interface" set type managed
        ip link set "$interface" up
        echo "Monitor mode disabled on $interface"
    fi
}

# Function to bring an interface up
bring_up() {
    interface=$(get_interfaces | gum filter --placeholder "Select interface to bring up")
    if [ -n "$interface" ]; then
        gum spin --spinner dot --title "Bringing up $interface..." -- sleep 2
        ip link set "$interface" up
        echo "$interface is up"
    fi
}

# Function to bring an interface down
bring_down() {
    interface=$(get_interfaces | gum filter --placeholder "Select interface to bring down")
    if [ -n "$interface" ]; then
        gum spin --spinner dot --title "Bringing down $interface..." -- sleep 2
        ip link set "$interface" down
        echo "$interface is down"
    fi
}

# Function to restart NetworkManager
restart_network_manager() {
    gum spin --spinner dot --title "Restarting NetworkManager..." -- systemctl restart NetworkManager
    echo "NetworkManager restarted"
}

# Function to rename an interface
rename_interface() {
    interface=$(get_interfaces | gum filter --placeholder "Select interface to rename")
    if [ -n "$interface" ]; then
        new_name=$(gum input --placeholder "Enter new name for $interface")
        if [ -n "$new_name" ]; then
            gum spin --spinner dot --title "Renaming $interface to $new_name..." -- sleep 2
            ip link set "$interface" down
            ip link set "$interface" name "$new_name"
            ip link set "$new_name" up
            echo "Interface $interface has been renamed to $new_name"
        fi
    fi
}

# Function to display the banner
show_banner() {
    gum style --foreground 212 "Welcome, Operator. SOZIN is online."
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣶⣄⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣠⣤⣶⣾⣿⣿⡇⠀"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣤⣤⡀⠀⠀⣠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠻⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣤⣤⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠉⠉⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⠀⠀⠀⢠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⢿⣿⣿⡟⠙⠻⣿⣿⡿⠿⠿⠟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⣿⡟⠀⠀⠀⢹⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⢿⣿⡀⠀⠀⠀⠹⣇⠀⠀⠀⣸⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠏⠀⠀⠀⠀⠻⣦⣄⣀⣉⣳⣦⡴⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 199 "⠿⢿⣿⣿⣿⣿⣿⣿⡿⠿⠿⠿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀"
    gum style --foreground 212 --bold "$(figlet -c -f big "SOZIN")"
}

# Main menu
main_menu() {
    while true; do
        clear
        show_banner
        choice=$(gum choose \
            "Enable Monitor Mode" \
            "Disable Monitor Mode" \
            "Bring Interface Up" \
            "Bring Interface Down" \
            "Rename Interface" \
            "Restart NetworkManager" \
            "Exit" --height 10 --cursor "=> " --cursor.foreground 212 --item.foreground 255)

        case "$choice" in
            "Enable Monitor Mode")
                enable_monitor_mode
                ;;
            "Disable Monitor Mode")
                disable_monitor_mode
                ;;
            "Bring Interface Up")
                bring_up
                ;;
            "Bring Interface Down")
                bring_down
                ;;
            "Rename Interface")
                rename_interface
                ;;
            "Restart NetworkManager")
                restart_network_manager
                ;;
            "Exit")
                exit
                ;;
        esac
        gum confirm "Do you want to perform another action?" || exit
    done
}

# Check for root and run main menu
check_root
main_menu
