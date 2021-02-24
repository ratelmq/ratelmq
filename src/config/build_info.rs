pub struct BuildInfo {
    pub version: &'static str,
    pub commit_hash: &'static str,
}

pub const BUILD_INFO: BuildInfo = BuildInfo {
    version: env!("CARGO_PKG_VERSION"),
    commit_hash: env!("BUILD_GIT_HASH"),
};
