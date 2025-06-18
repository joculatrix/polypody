use iced::{widget::{button, container, slider}, Border };

use super::CONTROL_BUTTON_SIZE;

pub(super) fn list_item(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        text_color: palette.background.base.text,
        background: match status {
            button::Status::Hovered | button::Status::Pressed =>
                iced::Color::parse("#222024").map(|c| c.into()),
            _ => iced::Color::parse("#242226").map(|c| c.into()),
        },
        border: Border {
            color: palette.background.base.color,
            width: 1.0,
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
            button::Status::Hovered => {
                iced::Color {
                    a: 0.75,
                    ..palette.background.base.text.into()
                }
            },
            button::Status::Pressed => {
                iced::Color {
                    a: 0.60,
                    ..palette.background.base.text.into()
                }
            }
            _ => palette.background.base.text.into(),
        },
        ..button::Style::default()
    }
}

pub(super) fn play_button(
    theme: &iced::Theme,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    button::Style {
        background: Some(match status {
            button::Status::Hovered | button::Status::Pressed => {
                iced::Color {
                    a: 0.75,
                    ..palette.background.base.text
                }
                .into()
            }
            _ => palette.background.base.text.into(),
        }),
        text_color: palette.background.base.color.into(),
        border: iced::Border::rounded(iced::Border::default(), CONTROL_BUTTON_SIZE / 2)
            .width(if status == button::Status::Pressed { 2 } else { 0 }),
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

        button::Style { text_color, ..button::Style::default() }
    }
}

pub(super) fn clean_slider(
    theme: &iced::Theme,
    status: slider::Status
) -> slider::Style {
    let palette = theme.extended_palette();

    slider::Style {
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: match status {
                    slider::Status::Dragged | slider::Status::Hovered => 8.0,
                    _ => 0.0,
                }
            },
            background: palette.background.base.color.into(),
            border_width: 2.0,
            border_color: if status == slider::Status::Dragged {
                palette.primary.base.color
            } else {
                palette.background.base.text
            }
        },
        rail: slider::Rail {
            backgrounds: (
                match status {
                    slider::Status::Dragged | slider::Status::Hovered =>
                        palette.primary.base.color.into(),
                    _ => palette.background.base.text.into(),
                },
                palette.background.base.color.into()
            ),
            width: 8.0,
            border: Border {
                color: palette.background.base.text,
                width: 1.0,
                radius: 2.0.into(),
            }
        }
    }
}
