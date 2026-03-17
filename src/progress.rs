use std::collections::HashMap;

use bollard::models::CreateImageInfo;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct PullProgress {
    multi: MultiProgress,
    bars: HashMap<String, ProgressBar>,
    style_download: ProgressStyle,
    style_extract: ProgressStyle,
    style_status: ProgressStyle,
}

impl PullProgress {
    pub fn new() -> Self {
        let style_download = ProgressStyle::with_template(
            "{prefix:>12.cyan} [{bar:30.green/dim}] {bytes}/{total_bytes} {msg}",
        )
        .unwrap()
        .progress_chars("=> ");

        let style_extract = ProgressStyle::with_template(
            "{prefix:>12.magenta} [{bar:30.magenta/dim}] {bytes}/{total_bytes} {msg}",
        )
        .unwrap()
        .progress_chars("=> ");

        let style_status =
            ProgressStyle::with_template("{prefix:>12.blue} {msg}").unwrap();

        Self {
            multi: MultiProgress::new(),
            bars: HashMap::new(),
            style_download,
            style_extract,
            style_status,
        }
    }

    pub fn update(&mut self, info: &CreateImageInfo) {
        let id = match &info.id {
            Some(id) => id.clone(),
            None => {
                // Status messages without an id (e.g. "Pulling from ...")
                if let Some(status) = &info.status {
                    let pb = self
                        .bars
                        .entry("__status__".to_string())
                        .or_insert_with(|| {
                            let pb = self.multi.add(ProgressBar::new(0));
                            pb.set_style(self.style_status.clone());
                            pb.set_prefix("Status");
                            pb
                        });
                    pb.set_message(status.clone());
                }
                return;
            }
        };

        let short_id = if id.len() > 12 { &id[..12] } else { &id };
        let status = info.status.as_deref().unwrap_or("");

        // Clone styles up front to avoid borrow conflicts with get_or_insert
        let style = match status {
            "Downloading" => self.style_download.clone(),
            "Extracting" => self.style_extract.clone(),
            _ => self.style_status.clone(),
        };

        match status {
            "Downloading" => {
                let pb = self.get_or_insert(&id, short_id);
                pb.set_style(style);
                pb.set_prefix(format!("{short_id} dl"));
                if let Some(detail) = &info.progress_detail
                    && let (Some(current), Some(total)) = (detail.current, detail.total)
                {
                    pb.set_length(total as u64);
                    pb.set_position(current as u64);
                }
            }
            "Extracting" => {
                let pb = self.get_or_insert(&id, short_id);
                pb.set_style(style);
                pb.set_prefix(format!("{short_id} ex"));
                if let Some(detail) = &info.progress_detail
                    && let (Some(current), Some(total)) = (detail.current, detail.total)
                {
                    pb.set_length(total as u64);
                    pb.set_position(current as u64);
                }
            }
            "Pull complete" | "Already exists" => {
                if let Some(pb) = self.bars.get(&id) {
                    pb.set_style(style);
                    pb.set_prefix(short_id.to_string());
                    pb.finish_with_message(status.to_string());
                }
            }
            _ => {
                let pb = self.get_or_insert(&id, short_id);
                pb.set_style(style);
                pb.set_prefix(short_id.to_string());
                pb.set_message(status.to_string());
            }
        }
    }

    pub fn finish(&self) {
        for pb in self.bars.values() {
            pb.finish_and_clear();
        }
        self.multi.clear().ok();
    }

    fn get_or_insert(&mut self, id: &str, short_id: &str) -> &ProgressBar {
        let multi = &self.multi;
        let style = &self.style_status;
        self.bars.entry(id.to_string()).or_insert_with(|| {
            let pb = multi.add(ProgressBar::new(0));
            pb.set_style(style.clone());
            pb.set_prefix(short_id.to_string());
            pb
        })
    }
}
