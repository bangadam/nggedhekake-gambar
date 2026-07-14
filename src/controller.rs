use std::path::Path;

use crate::source::{LoadedSource, SourceLoader};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceState {
    Empty,
    Ready(LoadedSource),
    Error(String),
}

pub struct WorkspaceController<L> {
    loader: L,
    state: WorkspaceState,
}

impl<L: SourceLoader> WorkspaceController<L> {
    pub fn new(loader: L) -> Self {
        Self {
            loader,
            state: WorkspaceState::Empty,
        }
    }

    pub fn state(&self) -> &WorkspaceState {
        &self.state
    }

    pub fn select_source(&mut self, path: &Path) {
        self.state = match self.loader.load(path) {
            Ok(source) => WorkspaceState::Ready(source),
            Err(error) => WorkspaceState::Error(error.to_string()),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, path::PathBuf};

    use crate::source::{PreviewPixels, SourceError};

    use super::*;

    struct StubLoader {
        results: RefCell<HashMap<PathBuf, Result<LoadedSource, SourceError>>>,
    }

    impl SourceLoader for StubLoader {
        fn load(&self, path: &Path) -> Result<LoadedSource, SourceError> {
            self.results
                .borrow_mut()
                .remove(path)
                .expect("missing stub result")
        }
    }

    fn source(path: &str, name: &str) -> LoadedSource {
        LoadedSource {
            path: PathBuf::from(path),
            name: name.into(),
            format: "PNG",
            width: 40,
            height: 20,
            preview: PreviewPixels {
                width: 40,
                height: 20,
                rgba: vec![0; 40 * 20 * 4],
            },
        }
    }

    #[test]
    fn selecting_a_valid_source_enters_ready_state() {
        let path = PathBuf::from("/tmp/first image.png");
        let expected = source("/tmp/first image.png", "first image.png");
        let loader = StubLoader {
            results: RefCell::new(HashMap::from([(path.clone(), Ok(expected.clone()))])),
        };
        let mut controller = WorkspaceController::new(loader);

        controller.select_source(&path);

        assert_eq!(controller.state(), &WorkspaceState::Ready(expected));
    }

    #[test]
    fn replacing_a_source_retains_only_the_latest_selection() {
        let first_path = PathBuf::from("/tmp/first.png");
        let second_path = PathBuf::from("/tmp/second.png");
        let first = source("/tmp/first.png", "first.png");
        let second = source("/tmp/second.png", "second.png");
        let loader = StubLoader {
            results: RefCell::new(HashMap::from([
                (first_path.clone(), Ok(first)),
                (second_path.clone(), Ok(second.clone())),
            ])),
        };
        let mut controller = WorkspaceController::new(loader);

        controller.select_source(&first_path);
        controller.select_source(&second_path);

        assert_eq!(controller.state(), &WorkspaceState::Ready(second));
    }

    #[test]
    fn invalid_selection_replaces_ready_state_with_actionable_error() {
        let valid_path = PathBuf::from("/tmp/valid.png");
        let invalid_path = PathBuf::from("/tmp/animated.webp");
        let loader = StubLoader {
            results: RefCell::new(HashMap::from([
                (
                    valid_path.clone(),
                    Ok(source("/tmp/valid.png", "valid.png")),
                ),
                (invalid_path.clone(), Err(SourceError::AnimatedWebp)),
            ])),
        };
        let mut controller = WorkspaceController::new(loader);

        controller.select_source(&valid_path);
        controller.select_source(&invalid_path);

        assert_eq!(
            controller.state(),
            &WorkspaceState::Error("Animated WebP is not supported. Choose a still image.".into())
        );
    }
}
