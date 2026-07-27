use std::sync::mpsc;
use std::thread;

use windows::Win32::Foundation::S_FALSE;
use windows::Win32::System::Com::*;

use crate::error::app_error::AppError;

type Task = Box<dyn FnOnce() -> Result<Box<dyn std::any::Any + Send>, AppError> + Send>;

pub struct ComStaWorker {
    sender: mpsc::Sender<Task>,
    thread: Option<thread::JoinHandle<()>>,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ComStaWorker {
    pub fn spawn() -> Result<Self, AppError> {
        let (task_tx, task_rx) = mpsc::channel::<Task>();
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let thread_handle = thread::Builder::new()
            .name("com-sta-worker".into())
            .spawn(move || {
                // SAFETY: Initialize COM STA for this thread
                let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
                let com_initialized = hr.is_ok() || hr == S_FALSE;
                let needs_uninit = hr.is_ok();

                while !shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    match task_rx.recv() {
                        Ok(task) => {
                            let _ = task();
                        }
                        Err(_) => break,
                    }
                }

                if com_initialized && needs_uninit {
                    // SAFETY: CoUninitialize for each successful CoInitializeEx
                    unsafe {
                        CoUninitialize();
                    }
                }
            })
            .map_err(|e| AppError::Internal(format!("failed to spawn COM STA worker: {}", e)))?;

        Ok(Self {
            sender: task_tx,
            thread: Some(thread_handle),
            shutdown,
        })
    }

    pub fn execute<T, F>(&self, task: F) -> Result<T, AppError>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, AppError> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let wrapped: Task = Box::new(move || {
            let result = task();
            match result {
                Ok(val) => {
                    let _ = tx.send(val);
                    Ok(Box::new(()) as Box<dyn std::any::Any + Send>)
                }
                Err(e) => Err(e),
            }
        });

        self.sender
            .send(wrapped)
            .map_err(|_| AppError::Internal("COM STA worker channel closed".into()))?;

        rx.recv()
            .map_err(|_| AppError::Internal("COM STA worker response channel closed".into()))
    }

    pub fn shutdown(&self) -> Result<(), AppError> {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Relaxed);
        // channel drop will cause thread to exit
        Ok(())
    }
}

impl Drop for ComStaWorker {
    fn drop(&mut self) {
        let _ = self.shutdown();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}
