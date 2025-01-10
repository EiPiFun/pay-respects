pub(crate) mod rust_i18n {
    use std::{collections::HashMap, sync::{LazyLock, RwLock, RwLockReadGuard}, ops::Deref};

    const RAW: &str = include_str!("../i18n/i18n.toml");
    static ROOT: LazyLock<serde_yml::Value> = LazyLock::new(|| serde_yml::from_str(RAW).unwrap());
    static STRINGS: LazyLock<HashMap<&str, HashMap<&str, &str>>> = LazyLock::new(|| {
        ROOT.as_mapping().unwrap().into_iter().map(|(k, v)| (k.as_str().unwrap(), v))
            .filter(|(k, _)| !k.starts_with('_'))
            .map(|(k, v)| (
                k,
                v.as_mapping().unwrap().into_iter()
                    .map(|(k, v)| (k.as_str().unwrap(), v.as_str().unwrap()))
                    .collect()
            ))
            .collect()
    });
    const FALLBACK_LOCALE: &str = "en";
    //pub static AVAILABLE_LOCALES: LazyLock<Vec<&str>> = LazyLock::new(|| STRINGS.values().nth(0).unwrap().keys().map(|&k| k).collect());
    static CURRENT_LOCALE: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(FALLBACK_LOCALE.to_string()));

    struct GuardedStr<'g>(RwLockReadGuard<'g, String>);

    impl Deref for GuardedStr<'_> {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.0.as_str()
        }
    }

    pub fn locale() -> impl Deref<Target = str> {
        GuardedStr(CURRENT_LOCALE.read().unwrap())
    }

    pub fn set_locale(locale: &str) {
        let mut l = CURRENT_LOCALE.write().unwrap();
        *l = locale.to_string();
    }

    pub fn get_string<'s>(locale: &str, key: &'s str) -> &'s str {
        *STRINGS.get(key).and_then(|t| t.get(locale)).unwrap_or(&key)
    }

    #[macro_export]
    macro_rules! t {
        ($key:expr) => {
            {
                let locale: &str = &crate::locale::rust_i18n::locale();
                std::borrow::Cow::Borrowed(crate::locale::rust_i18n::get_string(locale, $key))
            }
        };
        ($key:expr, $($kt:ident = $kv:expr),+) => {
            {
                let string = t!($key);
                $(
                    let string = string.replace(concat!("%{", stringify!($kt), "}"), &$kv.to_string());
                )+
                string
            }
        };
        ($key:expr, $($kt:literal => $kv:expr),+) => {
            {
                let string = t!($key);
                $(
                    let string = string.replace(concat!("%{", $kt, "}"), &$kv.to_string());
                )+
                string
            }
        };
    }
}
