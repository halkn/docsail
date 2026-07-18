use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum WorkspaceTarget {
    File(PathBuf),
    Directory(PathBuf),
}

impl WorkspaceTarget {
    pub fn path(&self) -> &Path {
        match self {
            Self::File(path) | Self::Directory(path) => path,
        }
    }
}

#[derive(Debug)]
pub enum WorkspaceError {
    NotFound(PathBuf),
    Unsupported(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(formatter, "path does not exist: {}", path.display()),
            Self::Unsupported(path) => write!(
                formatter,
                "path is neither a regular file nor a directory: {}",
                path.display()
            ),
            Self::Io { path, source } => {
                write!(formatter, "could not inspect {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn resolve(
    path: Option<&Path>,
    current_directory: &Path,
) -> Result<WorkspaceTarget, WorkspaceError> {
    let path = match path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => current_directory.join(path),
        None => {
            let docs_directory = current_directory.join("docs");

            if docs_directory.is_dir() {
                docs_directory
            } else {
                current_directory.to_path_buf()
            }
        }
    };

    classify(path)
}

fn classify(path: PathBuf) -> Result<WorkspaceTarget, WorkspaceError> {
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(WorkspaceError::NotFound(path));
        }
        Err(source) => return Err(WorkspaceError::Io { path, source }),
    };

    let canonical_path = fs::canonicalize(&path).map_err(|source| WorkspaceError::Io {
        path: path.clone(),
        source,
    })?;

    if metadata.is_file() {
        Ok(WorkspaceTarget::File(canonical_path))
    } else if metadata.is_dir() {
        Ok(WorkspaceTarget::Directory(canonical_path))
    } else {
        Err(WorkspaceError::Unsupported(canonical_path))
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkspaceError, WorkspaceTarget, resolve};
    use std::{
        fs,
        path::{Path, PathBuf},
        process,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_TEST_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let number = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("docsail-workspace-test-{}-{number}", process::id()));
            fs::create_dir_all(&path).unwrap();

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).unwrap();
        }
    }

    #[test]
    fn resolves_an_explicit_file_relative_to_the_current_directory() {
        let directory = TestDirectory::new();
        let file = directory.path().join("README.md");
        fs::write(&file, "# DocSail").unwrap();

        let target = resolve(Some(Path::new("README.md")), directory.path()).unwrap();

        assert_eq!(
            target,
            WorkspaceTarget::File(fs::canonicalize(file).unwrap())
        );
    }

    #[test]
    fn resolves_an_explicit_directory() {
        let directory = TestDirectory::new();
        let workspace = directory.path().join("workspace");
        fs::create_dir(&workspace).unwrap();

        let target = resolve(Some(Path::new("workspace")), directory.path()).unwrap();

        assert_eq!(
            target,
            WorkspaceTarget::Directory(fs::canonicalize(workspace).unwrap())
        );
    }

    #[test]
    fn prefers_docs_directory_when_no_path_is_given() {
        let directory = TestDirectory::new();
        let docs = directory.path().join("docs");
        fs::create_dir(&docs).unwrap();

        let target = resolve(None, directory.path()).unwrap();

        assert_eq!(target.path(), fs::canonicalize(docs).unwrap());
    }

    #[test]
    fn falls_back_to_the_current_directory_when_docs_is_missing() {
        let directory = TestDirectory::new();

        let target = resolve(None, directory.path()).unwrap();

        assert_eq!(target.path(), fs::canonicalize(directory.path()).unwrap());
    }

    #[test]
    fn reports_a_missing_explicit_path() {
        let directory = TestDirectory::new();

        let error = resolve(Some(Path::new("missing.md")), directory.path()).unwrap_err();

        assert!(
            matches!(error, WorkspaceError::NotFound(path) if path == directory.path().join("missing.md"))
        );
    }
}
