#[cfg(feature = "desktop-tauri")]
use license_control_suite::desktop::tauri::auth_command_handler;

#[cfg(feature = "desktop-tauri")]
pub fn app_invoke_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    auth_command_handler::<R>()
}

#[cfg(feature = "desktop-tauri")]
pub fn configure_app_builder<R>(builder: tauri::Builder<R>) -> tauri::Builder<R>
where
    R: tauri::Runtime,
{
    builder.invoke_handler(app_invoke_handler::<R>())
}

fn main() {
    println!("license-control-suite skeleton");
}
