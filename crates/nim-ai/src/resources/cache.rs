use std::path::PathBuf;

/// Where downloaded model weights live: `$NIM_CACHE`, else a `.cache/` directory at the root of
/// the enclosing repository, else `hf-hub`'s own default (`$HF_HOME`, else `~/.cache/huggingface`).
///
/// The fallback matters: a published `nim-ai` may run with no repository above it at all.
pub fn dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("NIM_CACHE") {
        return Some(PathBuf::from(path));
    }

    let current = std::env::current_dir().ok()?;
    let current = if let Some(git) = current.ancestors().find(|dir| dir.join(".git").exists()) {
        Some(git.to_path_buf())
    } else {
        current
            .ancestors()
            .filter(|dir| dir.join("Cargo.toml").exists())
            .last()
            .map(PathBuf::from)
    };

    current.map(|root| root.join(".cache"))
}
