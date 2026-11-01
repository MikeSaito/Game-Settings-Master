use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static STORE: Mutex<()> = Mutex::new(());

pub(super) struct StoreGuard {
    // Keep the OS lock until after all reads and the atomic replacement finish.
    _file: fs::File,
    _thread: MutexGuard<'static, ()>,
}

pub(super) fn lock_store() -> Result<StoreGuard, String> {
    let thread = STORE.lock().map_err(|error| error.to_string())?;
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(app_data_dir()?.join("profiles.lock"))
        .map_err(|error| error.to_string())?;
    // A stalled second instance must not leave startup/IPC waiting indefinitely.
    file.try_lock().map_err(|error| {
        format!(
            "{}: {error}",
            crate::i18n::t(
                "Хранилище профилей занято или недоступно",
                "Profile store is busy or unavailable",
            )
        )
    })?;
    Ok(StoreGuard {
        _file: file,
        _thread: thread,
    })
}

#[cfg(test)]
thread_local! {
    static TEST_APP_DATA_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) struct TestAppDataGuard {
    previous: Option<PathBuf>,
}

#[cfg(test)]
impl Drop for TestAppDataGuard {
    fn drop(&mut self) {
        TEST_APP_DATA_DIR.with(|slot| *slot.borrow_mut() = self.previous.take());
    }
}

#[cfg(test)]
pub(crate) fn use_test_app_data_dir(path: impl Into<PathBuf>) -> TestAppDataGuard {
    let previous = TEST_APP_DATA_DIR.with(|slot| slot.borrow_mut().replace(path.into()));
    TestAppDataGuard { previous }
}

#[cfg(test)]
fn test_app_data_dir() -> PathBuf {
    TEST_APP_DATA_DIR
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("target")
                .join("test-app-data")
                .join("default")
        })
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    #[cfg(test)]
    let dir = test_app_data_dir();
    #[cfg(all(not(test), feature = "native-test"))]
    let dir = crate::native_test::root()?.join("app-data");
    #[cfg(all(not(test), not(feature = "native-test")))]
    let dir = dirs::data_dir()
        .ok_or_else(|| {
            crate::i18n::t(
                "Не удалось определить каталог AppData",
                "Failed to determine AppData directory",
            )
        })?
        .join("UESettingsMaster");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub(crate) fn profiles_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("games.json"))
}

pub(crate) fn overrides_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("overrides.json"))
}

pub(crate) fn write_json_atomic(path: &std::path::Path, content: &str) -> Result<(), String> {
    crate::fs_util::write_file_bytes_opts(path, content.as_bytes(), true)
}
