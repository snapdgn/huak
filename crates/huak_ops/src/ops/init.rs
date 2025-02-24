use super::init_git;
use crate::{
    dependency::Dependency,
    fs,
    metadata::{default_entrypoint_string, LocalMetadata},
    package::importable_package_name,
    Config, Error, HuakResult, WorkspaceOptions,
};
use std::str::FromStr;

pub fn init_app_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    init_lib_project(config, options)?;

    let workspace = config.workspace();
    let mut metadata = workspace.current_local_metadata()?;

    let as_dep = Dependency::from_str(metadata.metadata().project_name())?;
    let entry_point =
        default_entrypoint_string(&importable_package_name(as_dep.name())?);
    metadata
        .metadata_mut()
        .add_script(as_dep.name(), &entry_point);
    metadata.write_file()
}

pub fn init_lib_project(
    config: &Config,
    options: &WorkspaceOptions,
) -> HuakResult<()> {
    let workspace = config.workspace();

    // Create a metadata file or error if one already exists.
    let mut metadata = match workspace.current_local_metadata() {
        Ok(_) => return Err(Error::MetadataFileFound),
        Err(_) => {
            LocalMetadata::template(workspace.root().join("pyproject.toml"))
        }
    };

    if options.uses_git {
        init_git(&config.workspace_root)?;
    }

    let name = fs::last_path_component(&config.workspace_root)?;
    metadata.metadata_mut().set_project_name(name);
    metadata.write_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        metadata::{default_pyproject_toml_contents, PyProjectToml},
        ops::test_config,
        Verbosity,
    };
    use tempfile::tempdir;

    #[test]
    fn test_init_lib_project() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("mock-project")).unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };
        init_lib_project(&config, &options).unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();

        assert_eq!(
            metadata.to_string_pretty().unwrap(),
            default_pyproject_toml_contents("mock-project")
        );
    }

    #[test]
    fn test_init_app_project() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("mock-project")).unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(root, cwd, Verbosity::Quiet);
        let options = WorkspaceOptions { uses_git: false };

        init_app_project(&config, &options).unwrap();

        let ws = config.workspace();
        let metadata = ws.current_local_metadata().unwrap();
        let pyproject_toml = PyProjectToml::default();
        pyproject_toml.project.clone().unwrap().name =
            String::from("mock-project");

        assert_eq!(
            metadata.to_string_pretty().unwrap(),
            r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "mock-project"
version = "0.0.1"
description = ""
dependencies = []

[project.scripts]
mock-project = "mock_project.main:main"
"#
        );
    }
}
