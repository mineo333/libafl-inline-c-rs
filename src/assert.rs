use std::{fs, path::PathBuf, process::Command};

pub struct Assert {
    command: assert_cmd::Command,
    files_to_remove: Option<Vec<PathBuf>>,
}

impl Assert {
    pub(crate) fn new(command: Command, files_to_remove: Option<Vec<PathBuf>>) -> Self {
        Self {
            command: assert_cmd::Command::from_std(command),
            files_to_remove,
        }
    }

    pub fn assert(&mut self) -> assert_cmd::assert::Assert {
        self.command.assert()
    }

    /// Shortcut to `self.assert().success()`.
    pub fn success(&mut self) -> assert_cmd::assert::Assert {
        self.assert().success()
    }

    /// Shortcut to `self.assert().failure()`.
    pub fn failure(&mut self) -> assert_cmd::assert::Assert {
        self.assert().failure()
    }
}

impl Drop for Assert {
    fn drop(&mut self) {
        if let Some(files_to_remove) = &self.files_to_remove {
            for file in files_to_remove.iter() {
                fs::remove_file(file).expect(&format!("Failed to remove `{:?}`", file));
            }
        }
    }
}