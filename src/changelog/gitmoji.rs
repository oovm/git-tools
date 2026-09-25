//! gitmoji 前缀解析与 release 分组。

use std::collections::HashMap;
use std::sync::LazyLock;

/// release 参考稿分组键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// Features (`✨`, `🎨`, `🚀`).
    Features,
    /// Bug fixes (`🐛`, `🚑`, `🔥`).
    Fixes,
    /// Breaking changes (`💥`).
    Breaking,
    /// 其它维护类 gitmoji。
    Other,
}

static GITMOJI_SECTION: LazyLock<HashMap<&'static str, Section>> = LazyLock::new(|| {
    HashMap::from([
        ("✨", Section::Features),
        ("🎨", Section::Features),
        ("🚀", Section::Features),
        ("🐛", Section::Fixes),
        ("🚑", Section::Fixes),
        ("🔥", Section::Fixes),
        ("💥", Section::Breaking),
        ("♻️", Section::Other),
        ("🔧", Section::Other),
        ("📝", Section::Other),
        ("👷", Section::Other),
        ("🧹", Section::Other),
        ("⬆️", Section::Other),
        ("🧪", Section::Other),
        ("🔨", Section::Other),
        ("📦", Section::Other),
    ])
});

static KNOWN_GITMOJIS: [&str; 16] = [
    "✨", "🎨", "🚀", "🐛", "🚑", "🔥", "💥", "♻️", "🔧", "📝", "👷", "🧹", "⬆️", "🧪", "🔨", "📦",
];

/// 返回 subject 行首 gitmoji（若有）。
pub fn leading_gitmoji(subject: &str) -> Option<&'static str> {
    for emoji in KNOWN_GITMOJIS {
        if subject.starts_with(emoji) {
            let rest = subject.strip_prefix(emoji).unwrap_or(subject);
            let rest = rest.strip_prefix('\u{fe0f}').unwrap_or(rest);
            if rest.starts_with(' ') {
                return Some(emoji);
            }
        }
    }
    None
}

/// 去掉 subject 行首 gitmoji 与后续空白。
pub fn strip_gitmoji(subject: &str) -> String {
    if let Some(emoji) = leading_gitmoji(subject) {
        let rest = subject.strip_prefix(emoji).unwrap_or(subject);
        let rest = rest.strip_prefix('\u{fe0f}').unwrap_or(rest);
        return rest.trim_start().to_string();
    }
    subject.trim().to_string()
}

/// 将 gitmoji 映射到 release 分组。
pub fn section_for_gitmoji(gitmoji: Option<&str>) -> Section {
    gitmoji.and_then(|emoji| GITMOJI_SECTION.get(emoji).copied()).unwrap_or(Section::Other)
}

/// 分组标题与空节占位文案。
pub fn section_meta(section: Section) -> (&'static str, &'static str) {
    match section {
        Section::Features => ("## ✨ Features", "(none)"),
        Section::Fixes => ("## 🐛 Bug Fixes", "(none)"),
        Section::Breaking => ("## ⚠️ Breaking Changes", "(none)"),
        Section::Other => ("## 📝 Other", "(none)"),
    }
}

/// 按 release 顺序列出全部分组。
pub fn all_sections() -> [Section; 4] {
    [Section::Features, Section::Fixes, Section::Breaking, Section::Other]
}
