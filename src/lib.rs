use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::process::Command;

/* re-export the semver version */
pub use semver::Version;

use ctreg::regex;

pub mod configuration;
pub mod configuration_builder;
pub mod error;

use configuration::Configuration;
use error::{Error, VersionError};

pub trait FileOps {
    /// Create temporary file
    /// # Errors
    /// When temporary file creation fails
    fn create_temp_exact(filename: &str) -> std::io::Result<(tempfile::TempDir, PathBuf, File)>;

    /// Writes content to path
    /// # Errors
    /// When writing fails
    fn write_all(file: &mut File, data: &[u8]) -> std::io::Result<()>;

    /// Reads content from path
    /// # Errors
    /// When reading fails
    fn read_file(path: &std::path::Path) -> std::io::Result<Vec<u8>>;
}

pub enum TempFS {}

impl FileOps for TempFS {
    fn create_temp_exact(filename: &str) -> std::io::Result<(tempfile::TempDir, PathBuf, File)> {
        let temp_dir = tempfile::TempDir::new()?;
        let path = PathBuf::from(temp_dir.path()).join(filename);
        let file = File::create(&path)?;
        Ok((temp_dir, path, file))
    }

    fn write_all(file: &mut File, data: &[u8]) -> std::io::Result<()> {
        file.write_all(data)
    }

    fn read_file(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
        let output_file = File::open(path)?;
        let mut reader = BufReader::new(output_file);
        let mut output_content = vec![];
        reader.read_to_end(&mut output_content)?;
        Ok(output_content)
    }
}

pub type Compiler = AbstractCompiler<TempFS>;

pub struct AbstractCompiler<FS: FileOps> {
    _fs: core::marker::PhantomData<FS>,
}

impl<FS: FileOps> AbstractCompiler<FS> {
    /// Compiles the current configuration into a WebAssembly module using
    /// the [emscripten compiler](https://emscripten.org/).
    ///
    /// # Errors
    /// - If using the host's file system fails.
    /// - If compilation fails
    pub fn compile(configuration: &Configuration) -> Result<Vec<u8>, Error> {
        let file_name = match &configuration.filename {
            configuration::Filename::Unspecified => "rustc-to-wasm-source.rs",
            configuration::Filename::Configured(filename) => filename.as_str(),
        };

        let (source_parent_dir, input_path, mut input_source) =
            FS::create_temp_exact(file_name).map_err(Error::IO)?;

        let (out_parent_dir, output_path, _output_wasm) =
            FS::create_temp_exact("rustc-to-wasm-out.wasm").map_err(Error::IO)?;

        // Write into temp file
        FS::write_all(&mut input_source, configuration.source().as_bytes()).map_err(Error::IO)?;

        let mut command = configuration.as_command(&input_path, &output_path);
        let output = command.output().map_err(Error::IO)?;

        if !output.status.success() {
            return Err(Error::Unsuccesful(output));
        }

        // Read from temp file
        let output_content = FS::read_file(&output_path).map_err(Error::IO)?;

        drop(source_parent_dir);
        drop(out_parent_dir);

        Ok(output_content)
    }
}

regex! { RustcSemVerRegex = r"rustc (?<semver>.*) \(.*\)" }

impl Compiler {
    /// Yields the version of the `rustc` compiler as a semver struct.
    ///
    /// # Errors
    /// - If `rustc` is not installed on the host
    /// - If the version cannot be read from the command output
    pub fn version() -> Result<Version, VersionError> {
        // Invoke command to request emcc version
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(VersionError::IO)?;

        // If invocation failed, yield early
        if !output.status.success() {
            return Err(VersionError::InvocationNoSuccess(output));
        }

        // Parse command output to `String`
        let command_output =
            String::try_from(output.stdout).map_err(VersionError::AttemptReadStdOut)?;

        // Parse command ourput to matching semver specification regex
        let Some(semver) = RustcSemVerRegex::new().captures(&command_output) else {
            return Err(VersionError::RegexNoMatch(command_output));
        };

        // Parse matching regex specification to `Semver`
        Version::parse(semver.semver.content).map_err(VersionError::VersionParseFailed)
    }
}
