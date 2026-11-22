//! 左侧来源列表：一次导入的每个文件都是一行，选中行是唯一的强调色填充。
//!
//! 导入动作留在工具栏，这里只负责「换一个图标」与「移除」——
//! 列表上不放第二套主操作，读起来才像来源列表而不是工具栏。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::state::App;
use crate::types::*;

#[component]
pub fn SourceList() -> impl IntoView {
    let app = expect_context::<App>();
    let count = move || app.item_count();
    let empty = move || app.item_count() == 0;

    view! {
        <aside class="if-pane if-sidebar c-pane">
            <div class="if-pane__head">
                <span class="if-pane__title">"导入项"</span>
                <span class="if-count">{move || count().to_string()}</span>
                <Push />
                <IconButton
                    icon=IconName::TRASH
                    label="移除全部导入项"
                    size=Size::Sm
                    variant=IconButtonVariant::Plain
                    disabled=empty()
                    on_click=Callback::new(move |_| app.clear_items())
                />
            </div>

            <div class="if-list">
                <Show
                    when=move || !empty()
                    fallback=move || view! {
                        <div class="if-empty">
                            <Icon name=IconName::IMAGES size=22 />
                            <p>"拖入文件到窗口，"<br/>"或点工具栏「导入」"</p>
                        </div>
                    }
                >
                    <For each=move || app.items.get() key=|item| item.id.clone() let:item>
                        <SourceRow item=item />
                    </For>
                </Show>
            </div>
        </aside>
    }
}

#[component]
fn SourceRow(item: InputItem) -> impl IntoView {
    let app = expect_context::<App>();
    let id = item.id.clone();
    let id_for_click = id.clone();
    let id_for_key = id.clone();
    let id_for_remove = id.clone();
    let name = item.display_name.clone();
    let thumb = format!("data:image/png;base64,{}", item.thumbnail_png_base64);
    let size = format!("{}×{}", item.source_width, item.source_height);
    let kind = match item.file_type {
        InputFileType::Image => None,
        InputFileType::Exe => Some("EXE"),
        InputFileType::Lnk => Some("LNK"),
        InputFileType::Directory => Some("DIR"),
    };
    let has_warning = !item.warnings.is_empty();
    let warning = item.warnings.join("\n");
    let selected = Signal::derive({
        let id = id.clone();
        move || app.selected.get().as_deref() == Some(id.as_str())
    });

    view! {
        <div
            class=move || {
                if selected.get() {
                    "if-source if-source--active"
                } else {
                    "if-source"
                }
            }
            role="option"
            aria-selected=move || selected.get().to_string()
            tabindex="0"
            title=warning.clone()
            on:click=move |_| app.select_item(id_for_click.clone())
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                if ev.key() == "Enter" || ev.key() == " " {
                    ev.prevent_default();
                    app.select_item(id_for_key.clone());
                }
            }
        >
            <img class="if-source__thumb" src=thumb alt="" draggable="false" />
            <div class="if-source__text">
                <span class="if-source__name">{name.clone()}</span>
                <span class="if-source__meta">
                    {size.clone()}
                    {kind.map(|kind| view! { <span class="if-source__kind">{kind}</span> })}
                    {has_warning.then(|| view! {
                        <span class="if-source__warn" title=warning.clone()>"!"</span>
                    })}
                </span>
            </div>
            <IconButton
                icon=IconName::X
                label=format!("移除 {name}")
                size=Size::Sm
                variant=IconButtonVariant::Plain
                class="if-source__remove"
                on_click=Callback::new(move |_| app.remove_item(id_for_remove.clone()))
            />
        </div>
    }
}
