use everything_plugin::log::{debug, warn};

pub fn check(prerelease: bool) -> Result<ib_update::github::UpdateInfo, ib_update::github::Error> {
    debug!("check...");
    let mirror = option_env!("UPDATE_MIRROR").unwrap_or("");
    if mirror.is_empty() {
        warn!("empty mirror");
    }
    let info = ib_update::github::UpdateConfig::builder()
        .owner("Chaoses-Ib")
        .repo("IbEverythingExt")
        // .current_version("0.1")
        .current_version(env!("CARGO_PKG_VERSION"))
        .base_urls(vec![mirror.into(), "https://api.github.com".into()])
        .user_agent(concat!("IbEverythingExt/", env!("CARGO_PKG_VERSION")))
        .build_blocking()
        .check_prerelease(prerelease);
    debug!(?info);
    info
}
