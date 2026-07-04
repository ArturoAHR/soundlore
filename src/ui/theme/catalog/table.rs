use crate::ui::{
    theme::Theme,
    widgets::table::{Catalog, Style},
};

pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| Style {})
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}
