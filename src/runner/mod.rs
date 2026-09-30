pub mod parser;
pub mod process;

pub use parser::{ExtractedMetadata, LogKind, LogParser, ParsedLog};
pub use process::{detect_package_manager, PackageManager, ProcessRunner, RunnerEvent, RunnerStatus};
