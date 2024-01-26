#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use inputbot::KeybdKey::*;
use single_instance::SingleInstance;

unsafe fn toggle_hdr() {
    use windows_sys::Win32::{Devices::Display::*, Foundation::ERROR_SUCCESS};

    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;
    unsafe {
        let result = GetDisplayConfigBufferSizes(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count as *mut u32,
            &mut mode_count as *mut u32,
        );

        if result != ERROR_SUCCESS {
            return;
        }

        let mut paths = Vec::with_capacity(path_count as usize);
        let mut modes = Vec::with_capacity(mode_count as usize);

        let result = QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count as *mut u32,
            paths.as_mut_ptr(),
            &mut mode_count as *mut u32,
            modes.as_mut_ptr(),
            std::ptr::null_mut(),
        );

        if result != ERROR_SUCCESS {
            return;
        }

        paths.set_len(path_count as usize);
        modes.set_len(mode_count as usize);

        for mode in modes {
            if mode.infoType != DISPLAYCONFIG_MODE_INFO_TYPE_TARGET {
                continue;
            }

            let mut get_color_info = DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
                    size: std::mem::size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO>() as u32,
                    adapterId: mode.adapterId,
                    id: mode.id,
                },
                Anonymous: DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO_0 { value: 0 },
                colorEncoding: 0,
                bitsPerColorChannel: 0,
            };

            let result = DisplayConfigGetDeviceInfo(
                &mut get_color_info.header as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
            );

            if result as u32 != ERROR_SUCCESS {
                continue;
            }

            let supported_mask_get: u32 = 1 << 0;
            let enabled_mask_get: u32 = 1 << 1;

            let advanced_color_supported =
                (get_color_info.Anonymous.value & supported_mask_get) != 0;
            let advanced_color_enabled = (get_color_info.Anonymous.value & enabled_mask_get) != 0;

            if advanced_color_supported {
                let mut set_color_info = DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE,
                        size: std::mem::size_of::<DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE>() as u32,
                        adapterId: mode.adapterId,
                        id: mode.id,
                    },
                    Anonymous: DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_0 { value: 0 },
                };

                let enabled_mask_set: u32 = 1 << 0;
                if advanced_color_enabled {
                    set_color_info.Anonymous.value |= !enabled_mask_set;

                    let result = DisplayConfigSetDeviceInfo(
                        &mut set_color_info.header as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
                    );
                    if result as u32 == ERROR_SUCCESS {
                        return;
                    }
                } else {
                    // Toggle advanced color enable on
                    set_color_info.Anonymous.value |= enabled_mask_set;

                    let result = DisplayConfigSetDeviceInfo(
                        &mut set_color_info.header as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
                    );

                    if result as u32 == ERROR_SUCCESS {
                        return;
                    }
                }
            }
        }
    }
}

fn main() {
    let instance = SingleInstance::new("faaux_hdr_toggle").unwrap();
    if !instance.is_single() {
        return;
    }

    BKey.bind(|| {
        if LSuper.is_pressed() && LAltKey.is_pressed() {
            unsafe {
                toggle_hdr();
            }
        }
    });

    inputbot::handle_input_events();
}
