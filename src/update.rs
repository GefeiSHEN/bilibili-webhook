use regex::Regex;

use std::{
    env::var_os,
    fs::read_to_string,
    io::Result,
    process::{Child, Command, Stdio},
};

use log::info;

use crate::{
    config::Feed,
    models::{sqlite, Content, Rss, Source},
    writer,
};

pub fn update(feed: &Feed) {
    let connection = sqlite::open();

    loop {
        let url = &feed.url;

        let rss_result = Rss::new(url);

        match rss_result {
            Ok(rss) => {
                let channel = &rss.channel;
                // 订阅源
                let source = Source::query_where(&connection, url)
                    .unwrap_or_else(|_| Source::insert(&connection, url, &channel.title));

                let mut is_update = false;

                // 内容
                for item in &channel.item {
                    if let Some(link) = extract_bilibili_link(&item.description) {
                        if Content::query_where(&connection, &link).is_err() {
                            // 返回错误，说明数据库没有这个内容，所以要更新
                            info!("[{}] 更新了一个新视频：{}", &source.title, &item.title);
                            // 下载新视频
                            match download(&link, feed) {
                                Ok(output) => {
                                    writer::bilili(&source.title, &link);
                                    let out = output.wait_with_output().unwrap();
                                    let out = String::from_utf8_lossy(&out.stdout);
                                    for line in out.split('\n') {
                                        writer::bilili(&source.title, line);
                                    }
                                    info!("\"{}\" 下载成功", &item.title);
                                    // 下载成功才在数据库添加内容
                                    Content::insert(&connection, source.id, &link, &item.title);
                                    is_update = true;
                                }
                                Err(error) => {
                                    log::error!("{}", error);
                                }
                            }
                        }
                    }
                }

                if is_update {
                    info!("[{}] 已更新！", &source.title);
                } else {
                    info!("[{}] 没有更新！", &source.title);
                }

                // 线程休眠
                let interval = &feed.interval * 60;
                std::thread::sleep(std::time::Duration::from_secs(interval));
                    },
        Err(e) => {
            // 线程休眠
            let interval = &feed.interval * 60;
            std::thread::sleep(std::time::Duration::from_secs(interval));
        }
        }
    }
}

fn extract_bilibili_link(description: &str) -> Option<String> {
    if description.contains("//转发自: @") {
        None
    } else {
        let regex = Regex::new(r"https://www\.bilibili\.com/video/BV[a-zA-Z0-9]+").unwrap();
        regex.find(description).map(|m| m.as_str().to_string())
    }
}

fn download(url: &str, feed: &Feed) -> Result<Child> {
    let mut cmd = Command::new("yutto");
    let args = feed.option.split(' ');
    for arg in args {
        cmd.arg(arg);
    }

    // 先从本地读取 sessdata
    if let Ok(sessdata) = read_to_string("config/SESSDATA.txt") {
        cmd.arg("-c").arg(sessdata);
        // 如果本地不存在，则从系统环境变量获取 sessdata
    } else if let Some(sessdata) = var_os("SESSDATA") {
        cmd.arg("-c").arg(sessdata);
    }

    let download_dir = format!("downloads/{}", &feed.path);
    cmd.args(["-d", &download_dir]).arg(url).stdout(Stdio::piped()).spawn()
}
