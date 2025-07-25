#![allow(non_snake_case)]

use std::sync::OnceLock;

use everything_plugin::{
    PluginApp, PluginHandler, PluginHost,
    ipc::{IpcWindow, Version},
    log::{debug, error},
    plugin_main,
    serde::{Deserialize, Serialize},
    ui::{OptionsPage, winio::spawn},
};
use ib_matcher::matcher::{PinyinMatchConfig, RomajiMatchConfig};

use crate::{
    home::UpdateConfig, pinyin::PinyinSearchConfig, quick_select::QuickSelectConfig,
    romaji::RomajiSearchConfig,
};

#[macro_use]
extern crate rust_i18n;

i18n!(fallback = "en");

mod ffi;
mod home;
mod pinyin;
mod quick_select;
pub mod romaji;
pub mod search;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub search: search::config::SearchConfig,
    /// 拼音搜索
    pub pinyin_search: PinyinSearchConfig,
    #[serde(default)]
    pub romaji_search: RomajiSearchConfig,
    /// 快速选择
    pub quick_select: QuickSelectConfig,
    /// 更新
    pub update: UpdateConfig,
}

pub struct App {
    config: Config,
    ipc: OnceLock<(IpcWindow, Version)>,
    offsets: Option<sig::EverythingExeOffsets>,
    pinyin: Option<PinyinMatchConfig<'static>>,
    romaji: Option<RomajiMatchConfig<'static>>,
}

impl PluginApp for App {
    type Config = Config;

    fn new(config: Option<Self::Config>) -> Self {
        // Read config.yaml as default
        let mut config = config.unwrap_or_else(|| ffi::read_config(true).unwrap_or_default());
        if !env!("CARGO_PKG_VERSION_PRE").is_empty() {
            config.update.prerelease = Some(true);
        }

        let ipc = match PluginHost::ipc_window_from_main_thread() {
            Some(ipc) => {
                let version = ipc.get_version();
                debug!(ipc_version = ?version);
                (ipc, version).into()
            }
            None => OnceLock::new(),
        };

        let pinyin = &config.pinyin_search;
        let pinyin = if pinyin.enable {
            Some(
                PinyinMatchConfig::builder(pinyin.notations())
                    // .data(&app.pinyin_data)
                    // TODO
                    // .case_insensitive(app.config.pinyin_search.)
                    .allow_partial_pattern(pinyin.allow_partial_match.unwrap_or(false))
                    .build(),
            )
        } else {
            None
        };

        let romaji = &config.romaji_search;
        let romaji = if romaji.enable {
            Some(
                RomajiMatchConfig::builder()
                    .allow_partial_pattern(romaji.allow_partial_match)
                    .build(),
            )
        } else {
            None
        };

        Self {
            ipc,
            pinyin,
            romaji,
            config,
            offsets: match sig::EverythingExeOffsets::from_current_exe() {
                Ok(offsets) => {
                    debug!(?offsets);
                    Some(offsets)
                }
                Err(e) => {
                    error!(%e, "Failed to get offsets from current exe");
                    None
                }
            },
        }
    }

    fn start(&self) {
        use pinyin::PinyinSearchMode;

        let host = HANDLER.get_host().is_some();

        // Dirty fixes
        let mut config = self.config.clone();
        config.pinyin_search.enable |= config.romaji_search.enable;
        config.pinyin_search.mode = match self.config.pinyin_search.mode {
            PinyinSearchMode::Auto => PinyinSearchMode::Pcre2,
            mode => mode,
        };
        config.quick_select.result_list.terminal = self
            .config
            .quick_select
            .result_list
            .terminal_command()
            .into();
        let config = serde_json::to_string(&config).unwrap();

        let instance_name = HANDLER
            .instance_name()
            .unwrap_or_default()
            .encode_utf16()
            .chain([0, 0])
            .collect::<Vec<_>>();

        let args = ffi::StartArgs {
            host,
            config: config.as_ptr() as _,
            ipc_window: self.ipc.get().map(|w| w.0.hwnd()).unwrap_or_default(),
            instance_name: instance_name.as_ptr(),
        };
        unsafe { ffi::start(&args) };
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn into_config(self) -> Self::Config {
        self.config.clone()
    }
}

impl App {
    /// Avaliable after:
    /// - v1.4: `on_ipc_window_created()`
    /// - v1.5: `new()`
    pub fn version(&self) -> Version {
        unsafe { self.ipc.get().unwrap_unchecked() }.1
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { ffi::stop() };
    }
}

plugin_main!(App, {
    PluginHandler::builder()
        .name(env!("CARGO_CRATE_NAME"))
        .description(t!("description"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .link(env!("CARGO_PKG_HOMEPAGE"))
        .options_pages(vec![
            OptionsPage::builder()
                .name("IbEverythingExt")
                .load(spawn::<home::options::MainModel>)
                .build(),
            OptionsPage::builder()
                .name(t!("pinyin-search"))
                .load(spawn::<pinyin::options::MainModel>)
                .build(),
            OptionsPage::builder()
                .name(t!("romaji-search"))
                .load(spawn::<romaji::options::MainModel>)
                .build(),
            OptionsPage::builder()
                .name(t!("quick-select"))
                .load(spawn::<quick_select::options::MainModel>)
                .build(),
        ])
        .build()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(env!("CARGO_CRATE_NAME"), "IbEverythingExt");
    }

    #[test]
    fn yaml() {
        let yaml = include_str!("../../publish/IbEverythingExt/config.yaml");
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(
            format!("{:?}", config),
            format!("{:?}", {
                let mut config = Config::default();
                config.pinyin_search.allow_partial_match = Some(false);
                config.quick_select.result_list.terminal = "wt -d ${fileDirname}".into();
                config.update.prerelease = Some(false);
                config
            })
        );
    }
}
