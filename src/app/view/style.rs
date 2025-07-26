use iced::{
    Border,
    widget::{button, container, slider, text},
};

use super::CONTROL_BUTTON_SIZE;

pub(super) fn context_menu(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        border: iced::border::rounded(1)
            .color(palette.background.base.text)
            .width(1),
        shadow: iced::Shadow {
            offset: iced::Vector { x: 2.0, y: 2.0 },
            color: iced::Color::parse("#222024").unwrap(),
            ..iced::Shadow::default()
        },
        ..container::Style::default()
    }
}

pub(super) fn list_button(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: match status {
            button::Status::Hovered => {
                iced::Color::parse("#242226").map(|c| c.into())
            }
            button::Status::Pressed => {
                iced::Color::parse("#222024").map(|c| c.into())
            }
            _ => Some(palette.background.base.color.into()),
        },
        text_color: match status {
            button::Status::Pressed => {
                palette.background.base.text.scale_alpha(0.75)
            }
            _ => palette.background.base.text,
        },
        ..button::Style::default()
    }
}

pub(super) fn control_panel_box(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: None,
        border: iced::border::rounded(5)
            .color(iced::Color {
                a: 0.10,
                ..palette.background.base.text
            })
            .width(2),
        ..container::Style::default()
    }
}

pub(super) fn dir_list_item(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        text_color: palette.background.base.text,
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                iced::Color::parse("#222024").map(|c| c.into())
            }
            _ => iced::Color::parse("#242226").map(|c| c.into()),
        },
        border: Border {
            color:  palette.background.base.color,
            width:  1.0,
            radius: 0.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn plain_icon_button(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        text_color: match status {
            button::Status::Hovered => iced::Color {
                a: 0.60,
                ..palette.background.base.text.into()
            },
            button::Status::Pressed => iced::Color {
                a: 0.40,
                ..palette.background.base.text.into()
            },
            _ => palette.background.base.text.into(),
        },
        background: Some(palette.background.base.color.into()),
        ..button::Style::default()
    }
}

pub(super) fn plain_icon_button_with_colors(
    background: Option<iced::Background>,
    text: Option<iced::Color>,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme: &iced::Theme, status: button::Status| {
        let palette = theme.extended_palette();
        let text = text.unwrap_or(palette.background.base.text);
        button::Style {
            text_color: match status {
                button::Status::Hovered => iced::Color { a: 0.60, ..text },
                button::Status::Pressed => iced::Color { a: 0.40, ..text },
                _ => text,
            },
            background,
            ..button::Style::default()
        }
    }
}

pub(super) fn play_button(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: Some(match status {
            button::Status::Hovered | button::Status::Pressed => iced::Color {
                a: 0.75,
                ..palette.background.base.text
            }
            .into(),
            _ => palette.background.base.text.into(),
        }),
        text_color: palette.background.base.color.into(),
        border: iced::Border::rounded(
            iced::Border::default(),
            CONTROL_BUTTON_SIZE / 2,
        )
        .width(if status == button::Status::Pressed {
            2
        } else {
            0
        }),
        ..button::Style::default()
    }
}

pub(super) fn toggle_icon_button(
    toggle_condition: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme: &iced::Theme, status| {
        let palette = theme.extended_palette();

        let text_color = {
            let color = if toggle_condition {
                palette.primary.base.color
            } else {
                palette.background.base.text
            };

            iced::Color {
                a: match status {
                    button::Status::Hovered => 0.75,
                    button::Status::Pressed => 0.60,
                    _ => 1.0,
                },
                ..color
            }
        };

        button::Style {
            text_color,
            ..button::Style::default()
        }
    }
}

pub(super) fn toggle_text(
    toggle_condition: bool,
) -> impl Fn(&iced::Theme) -> text::Style {
    move |theme: &iced::Theme| {
        let palette = theme.extended_palette();

        text::Style {
            color: Some(if toggle_condition {
                palette.primary.base.color
            } else {
                palette.background.base.text
            }),
        }
    }
}

pub(super) fn clean_slider(
    theme: &iced::Theme,
    status: slider::Status,
) -> slider::Style {
    let palette = theme.extended_palette();

    slider::Style {
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: match status {
                    slider::Status::Dragged | slider::Status::Hovered => 8.0,
                    _ => 4.0,
                },
            },
            background: match status {
                slider::Status::Dragged | slider::Status::Hovered => {
                    palette.background.base.color.into()
                }
                _ => palette.background.base.text.into(),
            },
            border_width: 2.0,
            border_color: if status == slider::Status::Dragged {
                palette.primary.base.color
            } else {
                palette.background.base.text
            },
        },
        rail:   slider::Rail {
            backgrounds: (
                match status {
                    slider::Status::Dragged | slider::Status::Hovered => {
                        palette.primary.base.color.into()
                    }
                    _ => palette.background.base.text.into(),
                },
                palette.background.base.color.into(),
            ),
            width: 8.0,
            border: Border {
                color:  palette.background.base.text,
                width:  1.0,
                radius: 2.0.into(),
            },
        },
    }
}

pub(super) fn outlined_button(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();
    button::Style {
        text_color: palette.background.base.text,
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                iced::Color::parse("#242226").map(|c| c.into())
            }
            _ => Some(palette.background.base.color.into()),
        },
        border: iced::Border {
            color:  palette.background.base.text.scale_alpha(0.2),
            width:  1.0,
            radius: (2.0).into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn bordered_container(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        border: iced::Border {
            color:  palette.background.base.text.scale_alpha(0.2),
            width:  1.0,
            radius: (2.0).into(),
        },
        ..container::Style::default()
    }
}

pub(super) fn track_list_container(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: iced::Color::parse("#242226").map(|c| c.into()),
        ..bordered_container(theme)
    }
}

pub(super) fn tracks_header(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.base.text.scale_alpha(0.6)),
        background: Some(palette.background.base.color.into()),
        ..container::Style::default()
    }
}
