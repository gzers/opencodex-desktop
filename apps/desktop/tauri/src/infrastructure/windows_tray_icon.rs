//! Windows uses its own transparent, monochrome notification-area artwork.
//! The system/taskbar theme, not the application's preference, selects its colour.

const DARK_ARTWORK: &[u8] = include_bytes!("../../icons/windows/tray-dark-32.png");
const LIGHT_ARTWORK: &[u8] = include_bytes!("../../icons/windows/tray-light-32.png");

fn artwork(system_is_light: bool) -> &'static [u8] {
    if system_is_light {
        DARK_ARTWORK
    } else {
        LIGHT_ARTWORK
    }
}

#[cfg(windows)]
pub(crate) fn install(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::Manager;
    app.manage(native::State::default());
    refresh(app)
}

#[cfg(windows)]
pub(crate) fn refresh(app: &tauri::AppHandle) -> tauri::Result<()> {
    native::refresh(app)
}

#[cfg(windows)]
mod native {
    use std::sync::Mutex;
    use tauri::Manager;
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};

    #[derive(Default)]
    pub(super) struct State(Mutex<Option<bool>>);

    fn system_is_light() -> bool {
        let path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
            .encode_utf16()
            .collect();
        let name: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();
        let mut value = 1_u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut value as *mut u32).cast(),
                &mut size,
            )
        };
        result != 0 || value != 0
    }

    pub(super) fn refresh(app: &tauri::AppHandle) -> tauri::Result<()> {
        let Some(state) = app.try_state::<State>() else {
            return Ok(());
        };
        let light = system_is_light();
        if state.0.lock().ok().is_some_and(|last| *last == Some(light)) {
            return Ok(());
        }
        let Some(tray) = app.tray_by_id(crate::modules::tray::TRAY_ID) else {
            return Ok(());
        };
        let image = tauri::image::Image::from_bytes(super::artwork(light))?;
        tray.set_icon(Some(image))?;
        tray.set_icon_as_template(false)?;
        if let Ok(mut last) = state.0.lock() {
            *last = Some(light);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_area_palette_follows_system_background() {
        assert_eq!(artwork(true), DARK_ARTWORK);
        assert_eq!(artwork(false), LIGHT_ARTWORK);
        assert_ne!(DARK_ARTWORK, LIGHT_ARTWORK);
    }
}
