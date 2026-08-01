The whole of §7.2's localization abstraction, assembled: a **key** and
**parameters** resolved against a **current locale**, through a **replaceable
provider**, with **fallback**, answering in a direction the **RTL** and **text
shaping** layers can use.

Everything it needs to be non-Anglocentric is held here rather than assumed:
the locale is a [`LanguageTag`] (default `und`, not `en`), the plural rules are
the *locale's* CLDR rules recomputed whenever the locale changes, and the
fallback chain ends at the root rather than at English.

# Example

```
# use obscur_localization::catalog::{Localizer, MessageCatalog};
# use obscur_localization::locale::LanguageTag;
# use obscur_localization::message::MessageArgs;
let ru = LanguageTag::parse("ru-RU").expect("valid tag");
let mut catalog = MessageCatalog::new();
catalog
    .insert(
        &LanguageTag::parse("ru").expect("valid tag"),
        "files",
        "{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
    )
    .expect("valid pattern");

let localizer = Localizer::new(ru).with_provider(Box::new(catalog));
let files = |n| localizer.format("files", &MessageArgs::new().with_integer("n", n));

assert_eq!(files(1), "1 файл");     // and 21, and 101
assert_eq!(files(2), "2 файла");    // and 22
assert_eq!(files(5), "5 файлов");   // and 11, and 111
```
