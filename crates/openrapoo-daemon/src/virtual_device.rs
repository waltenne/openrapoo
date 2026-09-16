//! uinput Virtual Device manager for OpenRapoo daemon.
//!
//! Creates a virtual input device via `/dev/uinput` to inject remapped mouse
//! clicks, keystrokes, and relative movement events into the kernel.

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, InputEvent, Key, RelativeAxisType,
};
use std::io;
use tracing::info;

/// Virtual input device wrapper for emitting remapped events.
pub struct OpenRapooVirtualDevice {
    device: VirtualDevice,
}

impl OpenRapooVirtualDevice {
    /// Create and register a new virtual device named "OpenRapoo Virtual Input Device".
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<Key>::new();
        // Register standard mouse buttons
        keys.insert(Key::BTN_LEFT);
        keys.insert(Key::BTN_RIGHT);
        keys.insert(Key::BTN_MIDDLE);
        keys.insert(Key::BTN_SIDE);
        keys.insert(Key::BTN_EXTRA);
        keys.insert(Key::BTN_FORWARD);
        keys.insert(Key::BTN_BACK);
        keys.insert(Key::BTN_TASK);

        // Register standard keyboard & media keys
        keys.insert(Key::KEY_LEFTCTRL);
        keys.insert(Key::KEY_RIGHTCTRL);
        keys.insert(Key::KEY_LEFTALT);
        keys.insert(Key::KEY_RIGHTALT);
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_RIGHTSHIFT);
        keys.insert(Key::KEY_LEFTMETA);
        keys.insert(Key::KEY_RIGHTMETA);

        keys.insert(Key::KEY_C);
        keys.insert(Key::KEY_V);
        keys.insert(Key::KEY_X);
        keys.insert(Key::KEY_Z);

        keys.insert(Key::KEY_PLAYPAUSE);
        keys.insert(Key::KEY_NEXTSONG);
        keys.insert(Key::KEY_PREVIOUSSONG);
        keys.insert(Key::KEY_MUTE);
        keys.insert(Key::KEY_VOLUMEUP);
        keys.insert(Key::KEY_VOLUMEDOWN);

        keys.insert(Key::KEY_F1);
        keys.insert(Key::KEY_F2);
        keys.insert(Key::KEY_F3);
        keys.insert(Key::KEY_F4);

        // Register all common keys up to KEY_MICMUTE to ensure macro compatibility
        for code in 1..255 {
            keys.insert(Key::new(code));
        }

        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_X);
        rel_axes.insert(RelativeAxisType::REL_Y);
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_HWHEEL);
        rel_axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);
        rel_axes.insert(RelativeAxisType::REL_HWHEEL_HI_RES);

        let mut device = VirtualDeviceBuilder::new()?
            .name("OpenRapoo Virtual Input Device")
            .with_keys(&keys)?
            .with_relative_axes(&rel_axes)?
            .build()?;

        info!("Created virtual input device: OpenRapoo Virtual Input Device");
        if let Ok(nodes) = device.enumerate_dev_nodes_blocking() {
            for node in nodes.flatten() {
                info!("Virtual device node available at: {}", node.display());
            }
        }

        Ok(OpenRapooVirtualDevice { device })
    }

    /// Emit a batch of events to the virtual device.
    pub fn emit(&mut self, events: &[InputEvent]) -> io::Result<()> {
        self.device.emit(events)
    }

    /// Emit a key press (value=1) followed immediately by a sync event.
    pub fn emit_key_down(&mut self, key_code: u16) -> io::Result<()> {
        let ev = InputEvent::new(evdev::EventType::KEY, key_code, 1);
        self.device.emit(&[ev])
    }

    /// Emit a key release (value=0) followed immediately by a sync event.
    pub fn emit_key_up(&mut self, key_code: u16) -> io::Result<()> {
        let ev = InputEvent::new(evdev::EventType::KEY, key_code, 0);
        self.device.emit(&[ev])
    }

    /// Emit a full click (down then up) for a key code.
    pub fn emit_click(&mut self, key_code: u16) -> io::Result<()> {
        self.emit_key_down(key_code)?;
        std::thread::sleep(std::time::Duration::from_millis(15));
        self.emit_key_up(key_code)
    }
}
