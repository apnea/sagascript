// Linux-specific platform code
//
// Linux doesn't have macOS-style accessibility permission gates or dock icons.
// Input simulation via uinput/X11/Wayland works without explicit user grants.

/// Linux doesn't have macOS-style accessibility permission gates.
/// Input simulation works without explicit user grants.
#[allow(dead_code)]
pub fn is_accessibility_trusted() -> bool {
    true
}

/// No-op on Linux — accessibility permissions aren't needed.
#[allow(dead_code)]
pub fn request_accessibility_permission() {
    // Nothing to do
}

/// No-op on Linux — tray-only behavior is handled by window manager
#[allow(dead_code)]
pub fn set_activation_policy_accessory() {
    // On Linux, window managers handle window visibility
}
