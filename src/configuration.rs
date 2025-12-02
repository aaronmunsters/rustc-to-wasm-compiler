use std::{path::Path, process::Command};

pub type Source = String;

trait IncludeInCommand {
    fn include_in(&self, command: &mut Command);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    O0,
    O1,
    O2,
    O3,
}

impl IncludeInCommand for Profile {
    fn include_in(&self, command: &mut Command) {
        let arg = match self {
            Profile::O0 => "-Copt-level=0",
            Profile::O1 => "-Copt-level=1",
            Profile::O2 => "-Copt-level=2",
            Profile::O3 => "-Copt-level=3",
        };

        command.arg(arg);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackSize {
    Unspecified,
    Configured(u32),
}

impl IncludeInCommand for StackSize {
    fn include_in(&self, command: &mut Command) {
        match self {
            StackSize::Unspecified => {}
            StackSize::Configured(size) => {
                command.arg(format!("-Clink-args=-zstack-size={size}"));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Debugging {
    Enabled,
    Disabled,
}

impl IncludeInCommand for Debugging {
    fn include_in(&self, command: &mut Command) {
        match self {
            Debugging::Enabled => {
                command.arg("-g");
            }
            Debugging::Disabled => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Configuration {
    pub(crate) profile: Profile,
    pub(crate) debugging: Debugging,
    pub(crate) stack_size: StackSize,
    pub(crate) source: String,
    pub(crate) filename: Filename,
}

#[derive(Clone, Debug)]
pub enum Filename {
    Unspecified,
    Configured(String),
}

impl Configuration {
    #[must_use]
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    #[must_use]
    pub fn debugging(&self) -> &Debugging {
        &self.debugging
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

impl Configuration {
    pub(crate) fn as_command(&self, input_path: &Path, output_path: &Path) -> Command {
        let mut command = Command::new("rustc");
        // Set output path
        command.arg(input_path);
        // Include performance profile
        self.profile.include_in(&mut command);
        // Include debug flag if set in configuration
        self.debugging.include_in(&mut command);
        // Include stack-size flag if set in configuration
        self.stack_size.include_in(&mut command);
        // Set wasm target
        command.arg("--target=wasm32-unknown-unknown");
        // Allow omitting a `main` function
        command.arg("--crate-type=cdylib");
        // Set output path
        command.arg("-o").arg(output_path);

        command
    }
}
