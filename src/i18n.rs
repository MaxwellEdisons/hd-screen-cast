use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    English,
    SimplifiedChinese,
}

struct Template {
    expression: Regex,
    english: String,
}

fn catalog() -> &'static HashMap<String, String> {
    static CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("locales/en.json"))
            .expect("valid built-in translation catalog")
    })
}

fn placeholder() -> &'static Regex {
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    PLACEHOLDER.get_or_init(|| Regex::new(r"\{[^{}]*\}").expect("valid placeholder pattern"))
}

fn templates() -> &'static Vec<Template> {
    static TEMPLATES: OnceLock<Vec<Template>> = OnceLock::new();
    TEMPLATES.get_or_init(|| {
        let mut entries: Vec<_> = catalog()
            .iter()
            .filter(|(source, _)| source.contains('{'))
            .collect();
        entries.sort_by_key(|(source, _)| std::cmp::Reverse(source.len()));
        entries
            .into_iter()
            .map(|(source, english)| {
                let mut pattern = String::from("(?s)^");
                let mut cursor = 0;
                for token in placeholder().find_iter(source) {
                    pattern.push_str(&regex::escape(&source[cursor..token.start()]));
                    pattern.push_str("(.*?)");
                    cursor = token.end();
                }
                pattern.push_str(&regex::escape(&source[cursor..]));
                pattern.push('$');
                Template {
                    expression: Regex::new(&pattern).expect("valid escaped translation pattern"),
                    english: english.clone(),
                }
            })
            .collect()
    })
}

pub fn tr(language: Language, source: impl AsRef<str>) -> String {
    if language == Language::SimplifiedChinese {
        return source.as_ref().to_owned();
    }
    translate(source.as_ref(), 0)
}

fn translate(source: &str, depth: u8) -> String {
    if depth > 5 {
        return source.to_owned();
    }
    if let Some(value) = catalog().get(source) {
        return value.clone();
    }
    for template in templates() {
        if let Some(values) = template.expression.captures(source) {
            let mut index = 0;
            return placeholder()
                .replace_all(&template.english, |_: &regex::Captures<'_>| {
                    index += 1;
                    translate(
                        values.get(index).map_or("", |value| value.as_str()),
                        depth + 1,
                    )
                })
                .into_owned();
        }
    }
    if source.contains('\n') {
        return source
            .lines()
            .map(|line| translate(line, depth + 1))
            .collect::<Vec<_>>()
            .join("\n");
    }
    source.to_owned()
}
