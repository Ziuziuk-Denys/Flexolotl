use flexolotl_core::settings::Language;

const RUSSIAN_READERS: [&str; 4] = ["ru", "uk", "be", "kk"];

pub fn resolve(language: Language) -> Language {
    match language {
        Language::System if system_prefers_russian() => Language::Russian,
        Language::System => Language::English,
        chosen => chosen,
    }
}

fn system_prefers_russian() -> bool {
    sys_locale::get_locale()
        .is_some_and(|locale| RUSSIAN_READERS.iter().any(|code| locale.starts_with(code)))
}
