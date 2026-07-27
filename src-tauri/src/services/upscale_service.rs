use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use crate::error::app_error::AppError;

/// Check whether the Real-CUGAN executable exists at the given path.
pub fn detect_realcugan(exe_path: &Path) -> bool {
    exe_path.is_file()
}

/// Run the Real-CUGAN upscaler on the input image.
///
/// Calls `realcugan-ncnn-vulkan.exe` with the given parameters. A timeout
/// of 120 seconds is enforced via a background thread and mpsc channel.
pub fn run_upscale(
    exe_path: &Path,
    input_path: &Path,
    output_path: &Path,
    scale: u32,
    denoise: i32,
    model_dir: &Path,
) -> Result<(), AppError> {
    let input_str = input_path
        .to_str()
        .ok_or_else(|| AppError::InvalidArgument("input path is not valid UTF-8".into()))?;
    let output_str = output_path
        .to_str()
        .ok_or_else(|| AppError::InvalidArgument("output path is not valid UTF-8".into()))?;
    let model_str = model_dir.to_str().ok_or_else(|| {
        AppError::InvalidArgument("model directory path is not valid UTF-8".into())
    })?;

    let mut child = Command::new(exe_path)
        .current_dir(exe_path.parent().unwrap_or_else(|| Path::new(".")))
        .arg("-i")
        .arg(input_str)
        .arg("-o")
        .arg(output_str)
        .arg("-s")
        .arg(scale.to_string())
        .arg("-n")
        .arg(denoise.to_string())
        .arg("-m")
        .arg(model_str)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            AppError::IoFailed(
                format!("failed to spawn {}: {}", exe_path.display(), e),
                None,
            )
        })?;

    let (tx, rx) = mpsc::channel();
    let child_pid = child.id();

    // Spawn a timeout thread
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(120));
        let _ = tx.send(child_pid);
    });

    // Wait for the child to finish or the timeout to fire
    let status = loop {
        match rx.try_recv() {
            Ok(_pid) => {
                // Timeout reached -- kill the process
                let _ = child.kill();
                return Err(AppError::Internal(
                    "Real-CUGAN upscale timed out after 120 seconds".into(),
                ));
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                // Channel closed, the timeout thread finished (unlikely)
                break child.wait();
            }
        }

        // Check if child has exited without blocking
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {
                // Still running, yield and check again
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => {
                break Err(e);
            }
        }
    };

    let status = status.map_err(|e| {
        AppError::IoFailed(
            format!("failed to wait for realcugan-ncnn-vulkan.exe: {}", e),
            None,
        )
    })?;

    if !status.success() {
        // Capture stderr for diagnostics
        let stderr_output = child
            .stderr
            .take()
            .and_then(|mut s| {
                let mut buf = String::new();
                s.read_to_string(&mut buf).ok().map(|_| buf)
            })
            .unwrap_or_default();

        let exit_code = status.code().unwrap_or(-1);
        let trimmed = stderr_output.trim();
        let detail = if trimmed.is_empty() {
            format!("exit code {}", exit_code)
        } else {
            format!("exit code {}: {}", exit_code, trimmed)
        };

        return Err(AppError::Internal(format!(
            "Real-CUGAN upscale failed: {}",
            detail
        )));
    }

    // Verify the output file was actually created
    if !output_path.is_file() {
        return Err(AppError::Internal(format!(
            "Real-CUGAN upscale completed but output file does not exist: {}",
            output_path.display()
        )));
    }

    Ok(())
}
