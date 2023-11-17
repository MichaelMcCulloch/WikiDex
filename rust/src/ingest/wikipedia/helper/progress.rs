use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
pub(crate) fn new_progress_bar(multibar: &MultiProgress, limit: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let pb = multibar.add(ProgressBar::new(limit));
    pb.set_style(sty);
    pb
}
