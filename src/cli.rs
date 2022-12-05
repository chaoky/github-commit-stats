use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub personal_token: Option<String>,
    #[arg(long)]
    pub cache_path: Option<String>,
    #[arg(short, long, value_enum, num_args = 0.., default_value = "programming")]
    pub categories: Vec<LanguageType>,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum LanguageType {
    Data,
    Markup,
    Programming,
    Prose,
}

impl From<hyperpolyglot::LanguageType> for LanguageType {
    fn from(val: hyperpolyglot::LanguageType) -> Self {
        match val {
            hyperpolyglot::LanguageType::Data => LanguageType::Data,
            hyperpolyglot::LanguageType::Markup => LanguageType::Markup,
            hyperpolyglot::LanguageType::Programming => LanguageType::Programming,
            hyperpolyglot::LanguageType::Prose => LanguageType::Prose,
        }
    }
}
