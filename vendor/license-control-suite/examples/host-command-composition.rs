#[cfg(feature = "desktop-tauri")]
use license_control_suite::desktop::tauri::{
    auth_command_handler, command_names,
};

#[cfg(feature = "desktop-tauri")]
pub fn composed_command_names() -> &'static [&'static str] {
    command_names()
}

#[cfg(feature = "desktop-tauri")]
pub fn build_host_owned_handler<R>(
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    auth_command_handler::<R>()
}

#[cfg(feature = "desktop-tauri")]
pub fn build_crate_owned_handler<R>(
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    auth_command_handler::<R>()
}

fn main() {}
