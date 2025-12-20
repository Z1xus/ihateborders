use anyhow::Result;
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::Win32::{
    Foundation::HWND,
    UI::Shell::{IsUserAnAdmin, ShellExecuteW},
    UI::WindowsAndMessaging::SW_SHOWNORMAL,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn is_elevated() -> bool
{
    unsafe { IsUserAnAdmin().as_bool() }
}

pub fn relaunch_elevated(args: &[&str]) -> Result<()>
{
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    let args_string = args.join(" ");

    unsafe {
        let operation = windows::core::w!("runas");
        let file = windows::core::HSTRING::from(exe_path_str.as_ref());
        let parameters = windows::core::HSTRING::from(&args_string);

        let result = ShellExecuteW(
            Some(HWND::default()),
            operation,
            &file,
            &parameters,
            None,
            SW_SHOWNORMAL,
        );

        if result.0 as i32 <= 32 {
            anyhow::bail!("Failed to relaunch with elevation");
        }
    }

    std::process::exit(0);
}

pub fn create_scheduled_task(with_admin: bool) -> Result<()>
{
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    let tr_arg = format!("\"{}\"", exe_path_str);

    let mut args = vec![
        "/Create",
        "/TN", "ihateborders_startup",
        "/TR", &tr_arg,
        "/SC", "ONLOGON",
        "/F",
    ];
    let rl_highest = "HIGHEST";
    if with_admin {
        args.push("/RL");
        args.push(rl_highest);
    }
    let output = Command::new("schtasks")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create scheduled task: {}", error);
    }

    Ok(())
}

pub fn remove_scheduled_task() -> Result<()>
{
    let output = Command::new("schtasks")
        .args(&[
            "/Delete",
            "/TN", "ihateborders_startup",
            "/F",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if !error.contains("cannot find the file") {
            anyhow::bail!("Failed to remove scheduled task: {}", error);
        }
    }

    Ok(())
}

pub fn task_exists() -> bool
{
    let output = Command::new("schtasks")
        .args(&[
            "/Query",
            "/TN", "ihateborders_startup",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}