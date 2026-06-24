use license_control_suite::desktop::tauri::{
    auth_command_handler, command_names, register_auth_commands,
};

pub fn build_host_owned_handler<R>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
where
    R: tauri::Runtime,
{
    auth_command_handler::<R>()
}

pub fn configure_host_builder<R>(builder: tauri::Builder<R>) -> tauri::Builder<R>
where
    R: tauri::Runtime,
{
    builder.invoke_handler(build_host_owned_handler::<R>())
}

pub fn configure_auth_only_builder<R>(builder: tauri::Builder<R>) -> tauri::Builder<R>
where
    R: tauri::Runtime,
{
    register_auth_commands(builder)
}

fn main() {
    assert_eq!(command_names().len(), 6);
}
