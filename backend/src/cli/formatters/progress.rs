use indicatif::{ProgressBar, ProgressStyle};

pub struct CliProgressBar {
    bar: ProgressBar,
}

impl CliProgressBar {
    pub fn new(total: usize) -> Self {
        let total = total.max(1) as u64;
        let bar = ProgressBar::new(total);
        let style = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("=> ");
        bar.set_style(style);
        bar.set_message("Searching providers");
        Self { bar }
    }

    pub fn set_total(&self, total: usize) {
        self.bar.set_length(total.max(1) as u64);
    }

    pub fn set_provider_phase(&self, completed_providers: usize, total_providers: usize) {
        self.bar.set_message(format!(
            "Searching providers ({}/{})",
            completed_providers, total_providers
        ));
    }

    pub fn set_discovery_phase(&self) {
        self.bar
            .set_message("Discovering relationships and compatibility signals");
    }

    pub fn advance(&self, processed: usize) {
        let total = self.bar.length().unwrap_or(1);
        self.bar.set_position((processed as u64).min(total));
    }

    pub fn finish(&self, relationships_discovered: usize) {
        self.bar.finish_with_message(format!(
            "Searching providers (\u{2713}), discovering relationships (\u{2713}, {} found)",
            relationships_discovered
        ));
    }
}
