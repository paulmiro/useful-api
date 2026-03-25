pub mod html;
pub mod markdown;
pub mod plaintext;
pub mod shell;

use pulldown_cmark::Options;

fn markdown_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}
