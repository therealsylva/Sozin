# SOZIN - Usage Examples

This document provides examples of how to use the `sozin` script to manage your network interfaces.

## 1. Main Menu

When you run `sudo sozin`, you will be greeted with the main menu, which displays all available options:

```
Welcome, Operator. SOZIN is online.
... (ASCII Art) ...

SOZIN

=> Enable Monitor Mode
   Disable Monitor Mode
   Bring Interface Up
   Bring Interface Down
   Rename Interface
   Restart NetworkManager
   Exit
```

You can navigate the menu with the arrow keys and press `Enter` to select an option.

## 2. Enabling Monitor Mode

This option allows you to switch a wireless interface to monitor mode, which is useful for network monitoring and packet sniffing.

1.  **Select "Enable Monitor Mode"** from the main menu.
2.  You will be prompted to **select an interface** from a list of available network devices.
3.  Once you select an interface, `sozin` will automatically bring it down, set it to monitor mode, and bring it back up.

## 3. Renaming an Interface

You can use this feature to give your network interfaces more descriptive names.

1.  **Select "Rename Interface"** from the main menu.
2.  **Choose the interface** you want to rename.
3.  You will be prompted to **enter a new name** for the interface.
4.  `sozin` will handle the process of taking the interface down, renaming it, and bringing it back up with the new name.

## 4. Restarting NetworkManager

If you're experiencing network issues, restarting the NetworkManager service can often resolve them.

1.  **Select "Restart NetworkManager"** from the main menu.
2.  `sozin` will immediately restart the service, and you'll see a confirmation message once it's done.

## 5. Exiting the Script

When you're finished managing your interfaces, you can exit the script in two ways:

-   Select the **"Exit"** option from the main menu.
-   After performing an action, you will be asked if you want to perform another. If you choose **"No"**, the script will exit.
