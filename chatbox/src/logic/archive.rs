use super::chat;
use crate::db::{self, data::SessionChats};
use crate::slint_generatedAppWindow::{AppWindow, ArchiveChatItem, ChatItem, Logic, Store};
use crate::util::translator::tr;
use log::warn;
use slint::Timer;
use slint::{ComponentHandle, Model, SortModel, VecModel};
use std::rc::Rc;
use std::time::Duration;
use uuid::Uuid;

pub fn init(ui: &AppWindow) {
    let ui_save_handle = ui.as_weak();
    let ui_delete_session_archive_handle = ui.as_weak();
    let ui_delete_session_archives_handle = ui.as_weak();
    let ui_show_handle = ui.as_weak();
    let ui_show_list_handle = ui.as_weak();
    let ui_reactive_handle = ui.as_weak();

    ui.global::<Logic>()
        .on_save_session_archive(move |suuid, name| {
            let ui = ui_save_handle.unwrap();
            let uuid = Uuid::new_v4().to_string();

            let mut datas: Vec<ArchiveChatItem> = ui
                .global::<Store>()
                .get_session_archive_datas()
                .iter()
                .collect();

            datas.push(ArchiveChatItem {
                name: name.as_str().into(),
                uuid: uuid.as_str().into(),
            });

            ui.global::<Store>()
                .set_session_archive_datas(Rc::new(VecModel::from(datas)).into());

            let chats: Vec<ChatItem> = ui.global::<Store>().get_session_datas().iter().collect();
            if chats.is_empty() {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}", tr("没有可归档的对话") + "！"),
                    "info".into(),
                );
            }

            match serde_json::to_string::<SessionChats>(&SessionChats::from(&chats)) {
                Ok(text) => {
                    if let Err(e) = match db::archive::is_table_exist(suuid.as_str()) {
                        Ok(true) => db::archive::insert(
                            suuid.as_str(),
                            uuid.as_str(),
                            name.as_str(),
                            text.as_str(),
                        ),
                        _ => match db::archive::new(suuid.as_str()) {
                            Ok(_) => db::archive::insert(
                                suuid.as_str(),
                                uuid.as_str(),
                                name.as_str(),
                                text.as_str(),
                            ),
                            Err(e) => Err(e),
                        },
                    } {
                        ui.global::<Logic>().invoke_show_message(
                            slint::format!("{}: {:?}", tr("保存失败") + "！" + &tr("原因"), e),
                            "warning".into(),
                        );
                    } else {
                        ui.global::<Logic>()
                            .invoke_show_message((tr("保存成功") + "!").into(), "success".into());
                    }
                }
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}: {:?}", tr("保存失败") + "！" + &tr("原因"), e),
                        "warning".into(),
                    );
                }
            }
        });

    ui.global::<Logic>()
        .on_delete_session_archive(move |suuid, uuid| {
            let ui = ui_delete_session_archive_handle.unwrap();
            let datas: Vec<ArchiveChatItem> = ui
                .global::<Store>()
                .get_session_archive_datas()
                .iter()
                .filter(|item| item.uuid != uuid)
                .collect();

            ui.global::<Store>()
                .set_session_archive_datas(Rc::new(VecModel::from(datas)).into());

            match db::archive::delete(suuid.as_str(), uuid.as_str()) {
                Err(e) => ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}: {:?}", tr("删除失败") + "!" + &tr("原因"), e),
                    "warning".into(),
                ),
                _ => ui
                    .global::<Logic>()
                    .invoke_show_message((tr("删除成功") + "!").into(), "success".into()),
            }
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_edit_session_archive(move |suuid, uuid, name| {
            let ui = ui_handle.unwrap();
            let datas = ui.global::<Store>().get_session_archive_datas();

            let (mut row, mut row_item) = (None, None);
            for (index, item) in datas.iter().enumerate() {
                if item.uuid == uuid {
                    row = Some(index);
                    row_item = Some(item);
                    break;
                }
            }

            if row.is_none() || row_item.is_none() {
                return;
            }

            let mut row_item = row_item.unwrap();
            row_item.name = name.clone();

            ui.global::<Store>().get_session_archive_datas()
                .set_row_data(row.unwrap(), row_item);

            if let Err(e) = db::archive::update(suuid.as_str(), uuid.as_str(), name.as_str()) {
                ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}: {:?}", tr("编辑失败") + "!" + &tr("原因"), e),
                    "warning".into(),
                );
                return;
            }
        });

    ui.global::<Logic>()
        .on_delete_session_archives(move |suuid| {
            let ui = ui_delete_session_archives_handle.unwrap();

            ui.global::<Store>()
                .set_session_archive_datas(Rc::new(VecModel::default()).into());

            match db::archive::drop_table(suuid.as_str()) {
                Err(e) => ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}: {:?}", tr("删除失败") + "!" + &tr("原因"), e),
                    "warning".into(),
                ),
                _ => ui
                    .global::<Logic>()
                    .invoke_show_message((tr("删除成功") + "!").into(), "success".into()),
            }
        });

    ui.global::<Logic>()
        .on_show_session_archive(move |suuid, uuid| {
            let ui = ui_show_handle.unwrap();

            match db::archive::is_table_exist(suuid.as_str()) {
                Ok(false) => return,
                Err(e) => {
                    ui.global::<Logic>().invoke_show_message(
                        slint::format!("{}: {:?}", tr("获取归档文件失败") + "!" + &tr("原因"), e),
                        "warning".into(),
                    );
                    return;
                }
                _ => (),
            }

            match db::archive::select(suuid.as_str(), uuid.as_str()) {
                Ok(Some(item)) => {
                    let data = item.1;
                    if data.is_empty() {
                        return;
                    }

                    match serde_json::from_str::<SessionChats>(&data) {
                        Ok(sc) => {
                            let chat_items = VecModel::default();
                            for citem in sc.chats.into_iter() {
                                chat_items.push(ChatItem {
                                    uuid: citem.uuid.into(),
                                    utext: citem.utext.as_str().into(),
                                    btext: citem.btext.as_str().into(),
                                    timestamp: citem.timestamp.into(),
                                    etext: "".into(),
                                    is_mark: citem.is_mark,
                                    is_hide: false,
                                    utext_items: chat::parse_chat_text(citem.utext.as_str()).into(),
                                    btext_items: chat::parse_chat_text(citem.btext.as_str()).into(),
                                })
                            }

                            ui.global::<Store>()
                                .set_session_datas(Rc::new(chat_items).into());

                            for i in 1..=3 {
                                let ui_handle = ui.as_weak();
                                Timer::single_shot(Duration::from_millis(i * 100), move || {
                                    let ui = ui_handle.unwrap();
                                    if f32::abs(ui.get_chats_viewport_y()) > 0.0 {
                                        ui.set_chats_viewport_y(0.0);
                                    }
                                });
                            }
                        }

                        Err(e) => ui.global::<Logic>().invoke_show_message(
                            slint::format!(
                                "{}: {:?}",
                                tr("获取归档文件失败") + "!" + &tr("原因"),
                                e
                            ),
                            "warning".into(),
                        ),
                    }
                }
                Ok(_) => (),
                Err(e) => ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}: {:?}", tr("获取归档文件失败") + "!" + &tr("原因"), e),
                    "warning".into(),
                ),
            }
        });

    ui.global::<Logic>()
        .on_show_session_archive_list(move |suuid, search_text| {
            let ui = ui_show_list_handle.unwrap();

            match db::archive::is_table_exist(suuid.as_str()) {
                Ok(true) => (),
                Err(e) => {
                    ui.global::<Store>()
                        .set_session_archive_datas(Rc::new(VecModel::default()).into());
                    warn!("{:?}", e);
                    return;
                }
                _ => {
                    ui.global::<Store>()
                        .set_session_archive_datas(Rc::new(VecModel::default()).into());
                    return;
                }
            }

            match db::archive::select_all(suuid.as_str()) {
                Ok(items) => {
                    // let search_text = ui.global::<Store>().get_archive_search_text();
                    let aitems = VecModel::default();
                    for item in items.into_iter() {
                        if search_text.is_empty() || item.1.contains(search_text.as_str()) {
                            aitems.push(ArchiveChatItem {
                                uuid: item.0.into(),
                                name: item.1.into(),
                            });
                        }
                    }

                    let aitems = SortModel::new(aitems, |a, b| {
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    });

                    ui.global::<Store>()
                        .set_session_archive_datas(Rc::new(aitems).into());
                }
                Err(e) => ui.global::<Logic>().invoke_show_message(
                    slint::format!("{}: {:?}", tr("获取归档文件失败") + "!" + &tr("原因"), e),
                    "warning".into(),
                ),
            }
        });

    ui.global::<Logic>()
        .on_archive_list_reactive(move |search_text| {
            let ui = ui_reactive_handle.unwrap();
            let search_text = search_text.trim_start();
            if search_text.is_empty() || !search_text.starts_with("/") {
                return;
            }

            ui.global::<Logic>().invoke_show_session_archive_list(
                ui.global::<Store>().get_current_session_uuid(),
                (&search_text[1..]).into(),
            );
        });
}
