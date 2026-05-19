use iced::widget::text::{Catalog, Style};

use crate::ui::theme::Theme;

pub type StyleFn<'a> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| Style {
            color: Some(theme.palette.text),
        })
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}
