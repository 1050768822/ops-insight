use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ops-insight", about = "New Relic 运维报告生成工具", version)]
pub struct Cli {
    #[arg(short, long, default_value = "config.toml", help = "配置文件路径")]
    pub config: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// 生成昨日运维报告
    Daily,

    /// 生成过去 7 天运维报告
    Weekly,

    /// 生成自定义时间范围报告
    Custom {
        #[arg(long, help = "开始时间 (YYYY-MM-DD)")]
        from: String,
        #[arg(long, help = "结束时间 (YYYY-MM-DD)")]
        to: String,
    },

    /// 读取本地 Serilog 日志（单个文件或文件夹）并分析
    Serilog {
        #[arg(short, long, help = "日志文件路径或文件夹路径")]
        path: String,

        #[arg(long, help = "只分析此日期之后的日志 (YYYY-MM-DD)，不填则分析全部")]
        from: Option<String>,

        #[arg(long, help = "只分析此日期之前的日志 (YYYY-MM-DD)，不填则分析全部")]
        to: Option<String>,
    },

    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 生成 config.toml 配置模板
    Init,
}
