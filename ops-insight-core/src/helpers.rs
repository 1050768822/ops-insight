use chrono::{DateTime, NaiveDate, Utc};

use crate::application::dtos::QueryRange;

pub fn parse_custom_range(from: &str, to: &str, hostnames: Vec<String>) -> anyhow::Result<QueryRange> {
    let from_dt = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("--from 格式错误，请使用 YYYY-MM-DD"))?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let to_dt = NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("--to 格式错误，请使用 YYYY-MM-DD"))?
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();
    Ok(QueryRange {
        from: from_dt,
        to: to_dt,
        hostnames,
    })
}

pub fn serilog_range(from: Option<&str>, to: Option<&str>) -> anyhow::Result<QueryRange> {
    let from_dt = match from {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("--from 格式错误，请使用 YYYY-MM-DD"))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        None => DateTime::from_timestamp(0, 0).unwrap(),
    };
    let to_dt = match to {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("--to 格式错误，请使用 YYYY-MM-DD"))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        None => Utc::now(),
    };
    Ok(QueryRange {
        from: from_dt,
        to: to_dt,
        hostnames: vec![],
    })
}

pub fn init_config() -> anyhow::Result<()> {
    let template = include_str!("../config.example.toml");
    if std::path::Path::new("config.toml").exists() {
        eprintln!("config.toml 已存在，跳过。");
    } else {
        std::fs::write("config.toml", template)?;
        println!("已生成 config.toml，请填写 API Key 后运行报告命令。");
    }
    Ok(())
}
