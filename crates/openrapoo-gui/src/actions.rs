//! GPUI Actions for OpenRapoo GUI navigation and events.

use gpui::actions;

actions!(
    openrapoo,
    [
        SelectDevice,
        BackToDevices,
        RefreshDevices,
        OpenActionEditor,
        CloseActionEditor,
    ]
);
