pub mod desktop_entries;
pub mod error;
pub mod filesystem;
pub mod polkit;
pub mod privilege;
pub mod process;

pub mod environment;

pub use desktop_entries::{DesktopEntry, DesktopEntryId};
pub use error::PlatformError;
pub use filesystem::SafePath;
pub use polkit::{
    AuthorizationResult, PolkitAction, PolkitClient, PolkitDetails, PolkitSubject, StubPolkitClient,
};
pub use privilege::{FlatpakAppId, FlatpakRemote, PrivilegedRequest, SystemdUnit};
pub use process::{
    KnownBinary, OutputLimits, ProcessEnv, ProcessOutput, ProcessSpec, ValidatedArg,
};
