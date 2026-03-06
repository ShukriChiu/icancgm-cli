/// Known data source identifiers.
pub const SOURCE_SINOCARE: &str = "sinocare";
pub const SOURCE_OURA: &str = "oura";
pub const SOURCE_DEXCOM: &str = "dexcom";
pub const SOURCE_LIBRE: &str = "libre";
pub const SOURCE_APPLE_HEALTH: &str = "apple-health";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    UserId,
    OAuthToken,
    Password,
    LocalExport,
}

#[derive(Debug, Clone)]
pub struct SourceMeta {
    pub name: &'static str,
    pub display_name: &'static str,
    pub auth_type: AuthType,
    pub guide_steps: &'static [&'static str],
    pub guide_url: Option<&'static str>,
    pub required_fields: &'static [&'static str],
}

impl SourceMeta {
    pub fn format_guide(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "📋 {} — 凭证获取指南\n\n",
            self.display_name
        ));

        let fields = self.required_fields.join(", ");
        out.push_str(&format!("  需要: {fields}\n\n"));

        out.push_str("  获取步骤:\n");
        for (i, step) in self.guide_steps.iter().enumerate() {
            out.push_str(&format!("  {}. {step}\n", i + 1));
        }

        if let Some(url) = self.guide_url {
            out.push_str(&format!("\n  📖 详情: {url}\n"));
        }

        out
    }
}

pub fn all_sources() -> &'static [SourceMeta] {
    &SOURCES
}

pub fn find_source(name: &str) -> Option<&'static SourceMeta> {
    SOURCES.iter().find(|s| s.name == name)
}

pub fn known_source_names() -> Vec<&'static str> {
    SOURCES.iter().map(|s| s.name).collect()
}

static SOURCES: [SourceMeta; 5] = [
    SourceMeta {
        name: SOURCE_SINOCARE,
        display_name: "三诺 CGM (Sinocare)",
        auth_type: AuthType::UserId,
        guide_steps: &[
            "打开「三诺爱看」App",
            "进入「我的」→「个人信息」",
            "找到「用户ID」或「UID」字段并复制",
            "运行: sino auth add sinocare --user-id <你的ID>",
        ],
        guide_url: None,
        required_fields: &["user_id"],
    },
    SourceMeta {
        name: SOURCE_OURA,
        display_name: "Oura Ring",
        auth_type: AuthType::OAuthToken,
        guide_steps: &[
            "登录 https://cloud.ouraring.com/personal-access-tokens",
            "点击「Create New Personal Access Token」",
            "给一个名字（如 \"sino-cli\"），勾选所需权限:\n     - daily: 日常数据（睡眠、活动、心率）\n     - heartrate: 心率数据\n     - sleep: 睡眠详情",
            "点击「Create」，复制生成的 token",
            "运行: sino auth add oura --token <你的token>",
            "⚠ Token 只显示一次，请妥善保管",
        ],
        guide_url: Some("https://cloud.ouraring.com/docs/authentication"),
        required_fields: &["token"],
    },
    SourceMeta {
        name: SOURCE_DEXCOM,
        display_name: "Dexcom CGM",
        auth_type: AuthType::Password,
        guide_steps: &[
            "确保你有 Dexcom Clarity 账号 (https://clarity.dexcom.com)",
            "如果没有，在 Dexcom App 中注册",
            "运行: sino auth add dexcom --username <邮箱> --password <密码>",
            "💡 也可以导出 CSV: 登录 Dexcom Clarity → 报告 → 导出",
        ],
        guide_url: Some("https://developer.dexcom.com"),
        required_fields: &["username", "password"],
    },
    SourceMeta {
        name: SOURCE_LIBRE,
        display_name: "Freestyle Libre (Abbott)",
        auth_type: AuthType::Password,
        guide_steps: &[
            "确保你有 LibreView 账号 (https://www.libreview.com)",
            "如果没有，通过 FreeStyle Libre App 注册",
            "运行: sino auth add libre --username <邮箱> --password <密码>",
            "💡 也可以导出 PDF/CSV: 登录 LibreView → 报告 → 导出",
        ],
        guide_url: Some("https://www.libreview.com"),
        required_fields: &["username", "password"],
    },
    SourceMeta {
        name: SOURCE_APPLE_HEALTH,
        display_name: "Apple Health",
        auth_type: AuthType::LocalExport,
        guide_steps: &[
            "打开 iPhone 上的「健康」App",
            "点击右上角头像 → 滑到底部 → 「导出所有健康数据」",
            "将导出的 export.zip 传到电脑上",
            "无需配置凭证，直接将数据粘贴到对话中即可",
        ],
        guide_url: None,
        required_fields: &[],
    },
];
