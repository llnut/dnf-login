// dnf-login: a launcher for Dungeon & Fighter written in Rust.
// Copyright (C) 2026 llnut
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*, Win32::NetworkManagement::IpHelper::*,
    Win32::System::Diagnostics::ToolHelp::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

#[cfg(target_os = "windows")]
pub fn get_mac_address() -> Result<String> {
    unsafe {
        let mut size: u32 = 0;
        let _ = GetAdaptersInfo(None, &mut size);

        if size == 0 {
            anyhow::bail!("No network adapters found");
        }

        let mut buffer = vec![0u8; size as usize];
        let adapter_info = buffer.as_mut_ptr() as *mut IP_ADAPTER_INFO;

        let result = GetAdaptersInfo(Some(adapter_info), &mut size);
        if result != NO_ERROR.0 {
            anyhow::bail!("Failed to get adapter info: {}", result);
        }

        // Find the first non-loopback adapter
        let mut current = adapter_info;
        while !current.is_null() {
            let adapter = &*current;

            if adapter.Type == 24 {
                // MIB_IF_TYPE_LOOPBACK
                current = adapter.Next;
                continue;
            }

            let mac_bytes = &adapter.Address[..adapter.AddressLength as usize];
            let mac_str = mac_bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join("-");

            return Ok(mac_str);
        }

        anyhow::bail!("No suitable network adapter found");
    }
}

#[cfg(target_os = "windows")]
pub fn is_process_running(process_name: &str) -> Result<bool> {
    let pids = find_process_pids(process_name)?;
    for &pid in &pids {
        if let Ok(handle) = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) } {
            let status = unsafe { WaitForSingleObject(handle, 0) };
            let _ = unsafe { CloseHandle(handle) };
            if status == WAIT_TIMEOUT {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(target_os = "windows")]
pub fn launch_dnf(
    token: &str,
    plugins_path: &str,
    inject_enabled: bool,
    server_ip: &str,
) -> Result<()> {
    let dnf_path = std::env::current_dir()?.join("DNF.exe");
    if !dnf_path.exists() {
        anyhow::bail!(
            "DNF.exe not found. Please place the launcher in the game directory.\nExpected path: {}",
            dnf_path.display()
        );
    }

    tracing::info!("Launching DNF with authentication token");

    let mut cmd = std::process::Command::new(&dnf_path);
    cmd.arg(token);
    if !server_ip.is_empty() {
        cmd.env("GAME_SERVER_IP", server_ip);
    }
    // Tell ijl15.dll whether to load plugins, and from which path.
    cmd.env("DNF_PLUGIN_ENABLED", if inject_enabled { "1" } else { "0" });
    if !plugins_path.is_empty() {
        cmd.env("DNF_PLUGIN_PATH", plugins_path);
    }
    let child = cmd.spawn()?;
    let pid = child.id();

    tracing::info!("DNF launched (PID: {})", pid);

    // Write launch diagnostics beside the launcher exe.
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let content = format!(
            "pid={}\ngame_server_ip={}\ninject_enabled={}\n",
            pid,
            if server_ip.is_empty() {
                "(none)"
            } else {
                server_ip
            },
            inject_enabled,
        );
        let _ = std::fs::write(dir.join("launch.log"), content);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn find_process_pids(process_name: &str) -> Result<Vec<u32>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        if snapshot.is_invalid() {
            anyhow::bail!("Failed to create process snapshot");
        }
        let mut pe32 = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut pids = Vec::new();
        let target = process_name.to_lowercase();
        if Process32FirstW(snapshot, &mut pe32).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &pe32.szExeFile[..pe32.szExeFile.iter().position(|&c| c == 0).unwrap_or(0)],
                );
                if name.to_lowercase() == target {
                    pids.push(pe32.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut pe32).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        Ok(pids)
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn close_window_by_pid(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let target_pid = lparam.0 as u32;
    let mut window_pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        if window_pid == target_pid {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    windows::core::BOOL(1)
}

/// Attempts to close the process gracefully via WM_CLOSE, then
/// force-terminates after 3 seconds if the process is still running.
#[cfg(target_os = "windows")]
pub fn graceful_terminate_process(process_name: &str) -> Result<()> {
    let pids = find_process_pids(process_name)?;
    if pids.is_empty() {
        return Ok(());
    }

    let handles: Vec<HANDLE> = pids
        .iter()
        .filter_map(|&pid| unsafe {
            OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_TERMINATE, false, pid).ok()
        })
        .collect();
    if handles.is_empty() {
        return Ok(());
    }

    for &pid in &pids {
        unsafe {
            let _ = EnumWindows(Some(close_window_by_pid), LPARAM(pid as isize));
        }
    }

    // WaitForMultipleObjects accepts at most 64 handles.
    let wait_slice = &handles[..handles.len().min(64)];
    let result = unsafe { WaitForMultipleObjects(wait_slice, true, 3000) };

    if result == WAIT_TIMEOUT || result == WAIT_FAILED {
        for &handle in &handles {
            unsafe {
                let _ = TerminateProcess(handle, 1);
            }
        }
        // TerminateProcess is guaranteed to eventually complete; wait
        // for the kernel to fully release all process resources.
        unsafe { WaitForMultipleObjects(wait_slice, true, INFINITE) };
    }

    for &handle in &handles {
        unsafe {
            let _ = CloseHandle(handle);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_mac_address() {
        match get_mac_address() {
            Ok(mac) => {
                assert!(!mac.is_empty());
                assert!(mac.contains('-'));
            }
            Err(e) => {
                tracing::warn!("Failed to get MAC address: {}", e);
            }
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_is_process_running() {
        let result = is_process_running("nonexistent_process_12345.exe");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let result = is_process_running("explorer.exe");
        if let Ok(running) = result {
            let _ = running;
        }
    }
}
