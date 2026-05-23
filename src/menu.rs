// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::menu::Item as MenuItem;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::{
    Element,
    app::Core,
    widget::{
        menu::{ItemHeight, ItemWidth},
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
    responsive_menu_bar()
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(320))
        .spacing(4.0)
        .into_element(
            core,
            key_binds,
            MENU_ID.clone(),
            Message::Surface,
            vec![(
                fl!("view"),
                vec![
                    MenuItem::Button(fl!("menu-settings"), None, Action::Settings),
                    MenuItem::Divider,
                    MenuItem::Button(fl!("menu-about"), None, Action::About),
                ],
            )],
        )
}
