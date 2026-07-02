// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Element,
    app::Core,
    theme,
    widget::{
        self, Widget,
        menu::{self, key_bind::KeyBind},
        responsive_menu_bar,
    },
};
use std::{collections::HashMap, sync::LazyLock};

use crate::{Action, Config, Message, fl};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

pub fn menu_bar<'a>(
    core: &Core,
    _config: &Config,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    menu::bar(vec![menu::Tree::with_children(
        widget::RcElementWrapper::new(
            widget::button::icon(widget::icon::from_name("open-menu-symbolic"))
                .padding([4, 12])
                .class(theme::Button::MenuRoot)
                .into(),
        ),
        menu::items(
            key_binds,
            vec![
                menu::Item::Button(fl!("menu-settings"), None, Action::Settings),
                menu::Item::Divider,
                menu::Item::Button(fl!("menu-about"), None, Action::About),
            ],
        ),
    )])
    .item_height(menu::ItemHeight::Dynamic(40))
    .item_width(menu::ItemWidth::Uniform(320))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .window_id_maybe(core.main_window_id())
    .on_surface_action(Message::Surface)
    .into()
}
