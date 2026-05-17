//! Kitchen-sink exercise of the imgui-rs surface.
//!
//! Drives every public builder/widget group in `arcdps_imgui` so that loading
//! this plugin under arcdps validates the bindings end-to-end against a host
//! that owns the imgui context, frame loop, and font atlas.
//!
//! Carve-outs (these would crash or corrupt the host and are intentionally not
//! exercised here): Context::create*, new_frame/end_frame/render, FontAtlas
//! mutation, Io/ConfigFlags writes, ini settings load/save.

use arcdps::imgui::{
    self, ChildFlags, Condition, Direction, DragDropFlags, ImColor32, InputTextFlags,
    SelectableFlags, SliderFlags, StyleColor, StyleVar, TableColumnSetup, TableFlags,
    TreeNodeFlags, Ui,
};
use std::sync::Mutex;

/// Persistent state for widgets that need it.
#[allow(dead_code)]
pub struct State {
    pub open: bool,

    // demo windows
    pub show_demo: bool,
    pub show_metrics: bool,
    pub show_about: bool,
    pub show_style_editor: bool,
    pub show_user_guide: bool,
    pub show_kitchen: bool,

    // text & input
    pub text_buf: String,
    pub multiline_buf: String,
    pub password_buf: String,
    pub hint_buf: String,
    pub int_val: i32,
    pub float_val: f32,
    pub vec2: [f32; 2],
    pub vec3: [f32; 3],
    pub vec4: [f32; 4],

    // numeric
    pub slider_f: f32,
    pub slider_i: i32,
    pub slider_arr: [f32; 3],
    pub vslider: f32,
    pub angle: f32,
    pub drag_f: f32,
    pub drag_arr: [i32; 4],
    pub drag_range_min: f32,
    pub drag_range_max: f32,

    // toggles
    pub checkbox: bool,
    pub flag_bits: u32,
    pub radio: i32,

    // color
    pub color3: [f32; 3],
    pub color4: [f32; 4],
    pub picker_color: [f32; 4],

    // selection
    pub combo_idx: usize,
    pub list_idx: usize,
    pub selectable_idx: i32,

    // table
    pub table_rows: Vec<TableRow>,

    // drag/drop
    pub dnd_slots: [u32; 3],

    // style
    pub style_alpha: f32,

    // tree
    pub tree_open: bool,

    // tab bar reorderable selection
    pub last_tab: String,
}

#[derive(Clone)]
pub struct TableRow {
    pub name: String,
    pub value: i32,
    pub ratio: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: true,
            show_demo: true,
            show_metrics: false,
            show_about: false,
            show_style_editor: false,
            show_user_guide: false,
            show_kitchen: true,
            text_buf: String::from("hello arcdps"),
            multiline_buf: String::from("line one\nline two\n"),
            password_buf: String::new(),
            hint_buf: String::new(),
            int_val: 42,
            float_val: 3.14,
            vec2: [1.0, 2.0],
            vec3: [1.0, 2.0, 3.0],
            vec4: [1.0, 2.0, 3.0, 4.0],
            slider_f: 0.5,
            slider_i: 50,
            slider_arr: [0.1, 0.2, 0.3],
            vslider: 0.0,
            angle: 0.0,
            drag_f: 0.0,
            drag_arr: [1, 2, 3, 4],
            drag_range_min: 0.0,
            drag_range_max: 1.0,
            checkbox: true,
            flag_bits: 0b0010,
            radio: 0,
            color3: [0.4, 0.7, 0.0],
            color4: [0.4, 0.7, 0.0, 1.0],
            picker_color: [0.2, 0.5, 0.9, 1.0],
            combo_idx: 0,
            list_idx: 0,
            selectable_idx: -1,
            table_rows: (0..8)
                .map(|i| TableRow {
                    name: format!("row {i}"),
                    value: i * 7,
                    ratio: (i as f32) / 8.0,
                })
                .collect(),
            dnd_slots: [0xff_00_00_ff, 0xff_00_ff_00, 0xff_ff_00_00],
            style_alpha: 1.0,
            tree_open: false,
            last_tab: String::new(),
        }
    }
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().expect("kitchen_sink state poisoned");
    let state = guard.get_or_insert_with(State::default);
    f(state)
}

/// Top-level entry; call from the arcdps `imgui` callback.
pub fn draw(ui: &Ui, _not_character_select_or_loading: bool) {
    with_state(|s| draw_inner(ui, s));
}

fn draw_inner(ui: &Ui, s: &mut State) {
    // Built-in demo windows: cover the C/cimgui surface.
    if s.show_demo {
        ui.show_demo_window(&mut s.show_demo);
    }
    if s.show_metrics {
        ui.show_metrics_window(&mut s.show_metrics);
    }
    if s.show_about {
        ui.show_about_window(&mut s.show_about);
    }
    if s.show_style_editor {
        ui.window("Style Editor")
            .opened(&mut s.show_style_editor)
            .build(|| ui.show_default_style_editor());
    }
    if s.show_user_guide {
        ui.window("User Guide")
            .opened(&mut s.show_user_guide)
            .build(|| ui.show_user_guide());
    }

    if !s.show_kitchen {
        return;
    }

    let mut open = s.show_kitchen;
    ui.window("imgui-rs kitchen sink")
        .opened(&mut open)
        .size([720.0, 540.0], Condition::FirstUseEver)
        .position([60.0, 60.0], Condition::FirstUseEver)
        .menu_bar(true)
        .build(|| {
            menu_bar(ui, s);
            tabs(ui, s);
        });
    s.show_kitchen = open;
}

fn menu_bar(ui: &Ui, s: &mut State) {
    if let Some(_token) = ui.begin_menu_bar() {
        ui.menu("Demo windows", || {
            ui.checkbox("Demo", &mut s.show_demo);
            ui.checkbox("Metrics", &mut s.show_metrics);
            ui.checkbox("About", &mut s.show_about);
            ui.checkbox("Style editor", &mut s.show_style_editor);
            ui.checkbox("User guide", &mut s.show_user_guide);
        });
        ui.menu("Help", || {
            if ui.menu_item("Reset state") {
                *s = State::default();
            }
            ui.separator();
            ui.menu_item_config("Disabled item").enabled(false).build();
        });
    }
}

fn tabs(ui: &Ui, s: &mut State) {
    if let Some(_tb) = ui.tab_bar("kitchen-tabs") {
        tab(ui, "Text", || tab_text(ui, s));
        tab(ui, "Input", || tab_input(ui, s));
        tab(ui, "Numeric", || tab_numeric(ui, s));
        tab(ui, "Buttons", || tab_buttons(ui, s));
        tab(ui, "Color", || tab_color(ui, s));
        tab(ui, "Selection", || tab_selection(ui, s));
        tab(ui, "Trees", || tab_trees(ui, s));
        tab(ui, "Tables", || tab_tables(ui, s));
        tab(ui, "Popups", || tab_popups(ui, s));
        tab(ui, "Tooltips", || tab_tooltips(ui, s));
        tab(ui, "DragDrop", || tab_dragdrop(ui, s));
        tab(ui, "Layout", || tab_layout(ui, s));
        tab(ui, "Stacks", || tab_stacks(ui, s));
        tab(ui, "Plots", || tab_plots(ui, s));
        tab(ui, "DrawList", || tab_drawlist(ui));
        tab(ui, "IO/Style", || tab_io_style(ui));
        tab(ui, "Clipboard", || tab_clipboard(ui, s));
    }
}

fn tab(ui: &Ui, name: &str, body: impl FnOnce()) {
    if let Some(_t) = ui.tab_item(name) {
        body();
    }
}

// ---------------------------------------------------------------- text

fn tab_text(ui: &Ui, _s: &mut State) {
    ui.text("plain text");
    ui.text_colored([1.0, 0.5, 0.2, 1.0], "colored text");
    ui.text_disabled("disabled text");
    ui.text_wrapped(
        "wrapped text — this is a long string that should wrap when the window is narrow enough to force line breaks within the available content region.",
    );
    ui.label_text("label", "value");
    ui.bullet();
    ui.same_line();
    ui.text("bullet via Ui::bullet");
    ui.bullet_text("bullet_text helper");
    ui.separator();
    ui.text(format!("frame_count = {}", ui.frame_count()));
}

// ---------------------------------------------------------------- input

fn tab_input(ui: &Ui, s: &mut State) {
    ui.input_text("single", &mut s.text_buf).build();
    ui.input_text("hint", &mut s.hint_buf)
        .hint("type here…")
        .build();
    ui.input_text("password", &mut s.password_buf)
        .flags(InputTextFlags::PASSWORD)
        .password(true)
        .build();
    ui.input_text_multiline("multi", &mut s.multiline_buf, [0.0, 80.0])
        .build();

    ui.separator();
    ui.input_int("int", &mut s.int_val).build();
    ui.input_float("float", &mut s.float_val).build();
    let mut v2 = s.vec2;
    if ui.input_float2("vec2", &mut v2).build() {
        s.vec2 = v2;
    }
    let mut v3 = s.vec3;
    if ui.input_float3("vec3", &mut v3).build() {
        s.vec3 = v3;
    }
    let mut v4 = s.vec4;
    if ui.input_float4("vec4", &mut v4).build() {
        s.vec4 = v4;
    }
}

// ---------------------------------------------------------------- numeric

fn tab_numeric(ui: &Ui, s: &mut State) {
    ui.slider("slider f32", 0.0_f32, 1.0_f32, &mut s.slider_f);
    ui.slider_config("slider i32", 0_i32, 100_i32)
        .flags(SliderFlags::ALWAYS_CLAMP)
        .build(&mut s.slider_i);
    ui.slider_config("slider [f32; 3]", 0.0_f32, 1.0_f32)
        .build_array(&mut s.slider_arr);

    let _ = imgui::VerticalSlider::new("vslider", [40.0, 120.0], 0.0_f32, 1.0_f32)
        .build(ui, &mut s.vslider);
    ui.same_line();
    imgui::AngleSlider::new("angle").build(ui, &mut s.angle);

    ui.separator();
    imgui::Drag::new("drag f32")
        .range(-10.0_f32, 10.0_f32)
        .speed(0.05)
        .build(ui, &mut s.drag_f);
    imgui::Drag::new("drag [i32; 4]").build_array(ui, &mut s.drag_arr);
    imgui::DragRange::new("drag range")
        .range(0.0_f32, 1.0_f32)
        .build(ui, &mut s.drag_range_min, &mut s.drag_range_max);

    ui.separator();
    imgui::ProgressBar::new(s.slider_f).size([200.0, 0.0]).build(ui);
    imgui::ProgressBar::new(s.slider_f)
        .overlay_text(format!("{:.0}%", s.slider_f * 100.0))
        .build(ui);
}

// ---------------------------------------------------------------- buttons

fn tab_buttons(ui: &Ui, s: &mut State) {
    if ui.button("button") {
        log::info!("button clicked");
    }
    ui.same_line();
    ui.button_with_size("sized", [120.0, 0.0]);
    ui.same_line();
    ui.small_button("small");

    ui.arrow_button("arrow_l", Direction::Left);
    ui.same_line();
    ui.arrow_button("arrow_r", Direction::Right);
    ui.same_line();
    ui.arrow_button("arrow_u", Direction::Up);
    ui.same_line();
    ui.arrow_button("arrow_d", Direction::Down);

    ui.invisible_button("invisible", [60.0, 30.0]);
    ui.same_line();
    ui.text("(invisible button →)");

    ui.separator();
    ui.checkbox("checkbox", &mut s.checkbox);
    ui.checkbox_flags("flag bit 0", &mut s.flag_bits, 0b0001);
    ui.checkbox_flags("flag bit 1", &mut s.flag_bits, 0b0010);
    ui.checkbox_flags("flag bit 2", &mut s.flag_bits, 0b0100);

    ui.radio_button("radio A", &mut s.radio, 0);
    ui.same_line();
    ui.radio_button("radio B", &mut s.radio, 1);
    ui.same_line();
    ui.radio_button("radio C", &mut s.radio, 2);
    ui.same_line();
    if ui.radio_button_bool("bool radio (always true)", true) {
        // no-op
    }

    ui.separator();
    let _disabled = ui.begin_disabled(true);
    ui.button("disabled button (token)");
    drop(_disabled);
    ui.disabled(true, || {
        ui.button("disabled button (closure)");
    });
}

// ---------------------------------------------------------------- color

fn tab_color(ui: &Ui, s: &mut State) {
    ui.color_edit3("color3", &mut s.color3);
    ui.color_edit4("color4", &mut s.color4);
    ui.color_picker4_config("picker4", &mut s.picker_color)
        .alpha(true)
        .small_preview(true)
        .side_preview(true)
        .build();
    let _ = ui.color_button("button", s.color4);
}

// ---------------------------------------------------------------- selection

fn tab_selection(ui: &Ui, s: &mut State) {
    let items = ["alpha", "beta", "gamma", "delta", "epsilon"];

    ui.combo_simple_string("combo", &mut s.combo_idx, &items);
    if let Some(_token) = ui.begin_combo("combo (manual)", items[s.combo_idx]) {
        for (i, name) in items.iter().enumerate() {
            let selected = i == s.combo_idx;
            if ui
                .selectable_config(*name)
                .selected(selected)
                .flags(SelectableFlags::empty())
                .build()
            {
                s.combo_idx = i;
            }
        }
    }

    imgui::ListBox::new("list").build(ui, || {
        for (i, name) in items.iter().enumerate() {
            if ui.selectable_config(*name).selected(i == s.list_idx).build() {
                s.list_idx = i;
            }
        }
    });

    ui.separator();
    for i in 0..5_i32 {
        if ui
            .selectable_config(format!("selectable {i}"))
            .selected(s.selectable_idx == i)
            .build()
        {
            s.selectable_idx = i;
        }
    }
}

// ---------------------------------------------------------------- trees

fn tab_trees(ui: &Ui, s: &mut State) {
    if let Some(_t) = ui.tree_node("simple") {
        ui.text("inside simple tree");
    }
    if let Some(_t) = ui
        .tree_node_config("configured")
        .label::<&str, _>("configured (default open)")
        .default_open(true)
        .framed(true)
        .push()
    {
        ui.text("inside framed tree");
        if let Some(_t2) = ui.tree_node("nested") {
            ui.text("nested leaf");
        }
    }

    if ui.collapsing_header("collapsing", TreeNodeFlags::empty()) {
        ui.text("collapsed content");
    }
    let mut closable = s.tree_open;
    if ui.collapsing_header_with_close_button(
        "closable",
        TreeNodeFlags::DEFAULT_OPEN,
        &mut closable,
    ) {
        ui.text("with close button");
    }
    s.tree_open = closable;
}

// ---------------------------------------------------------------- tables

fn tab_tables(ui: &Ui, s: &mut State) {
    let cols = [
        TableColumnSetup::new("name"),
        TableColumnSetup::new("value"),
        TableColumnSetup::new("ratio"),
    ];
    if let Some(_tok) = ui.begin_table_header_with_flags(
        "kitchen-table",
        cols,
        TableFlags::ROW_BG
            | TableFlags::BORDERS
            | TableFlags::RESIZABLE
            | TableFlags::SORTABLE
            | TableFlags::SCROLL_Y,
    ) {
        if let Some(specs) = ui.table_sort_specs_mut() {
            specs.conditional_sort(|s_specs| {
                if let Some(spec) = s_specs.iter().next() {
                    let col = spec.column_idx();
                    let ascending = matches!(
                        spec.sort_direction(),
                        Some(imgui::TableSortDirection::Ascending)
                    );
                    s.table_rows.sort_by(|a, b| {
                        let ord = match col {
                            0 => a.name.cmp(&b.name),
                            1 => a.value.cmp(&b.value),
                            _ => a
                                .ratio
                                .partial_cmp(&b.ratio)
                                .unwrap_or(std::cmp::Ordering::Equal),
                        };
                        if ascending { ord } else { ord.reverse() }
                    });
                }
            });
        }

        for row in &s.table_rows {
            ui.table_next_row();
            ui.table_next_column();
            ui.text(&row.name);
            ui.table_next_column();
            ui.text(format!("{}", row.value));
            ui.table_next_column();
            imgui::ProgressBar::new(row.ratio).build(ui);
        }
    }

    ui.separator();
    ui.text(format!(
        "table_column_count outside table: {}",
        ui.table_column_count()
    ));
}

// ---------------------------------------------------------------- popups

fn tab_popups(ui: &Ui, _s: &mut State) {
    if ui.button("open simple popup") {
        ui.open_popup("simple-popup");
    }
    ui.popup("simple-popup", || {
        ui.text("inside popup");
        if ui.button("close") {
            ui.close_current_popup();
        }
    });

    if ui.button("open modal") {
        ui.open_popup("modal-popup");
    }
    if let Some(_t) = ui.modal_popup_config("modal-popup").begin_popup() {
        ui.text("inside modal");
        if ui.button("ok") {
            ui.close_current_popup();
        }
    }

    ui.separator();
    ui.button("right-click me");
    if let Some(_t) = ui.begin_popup_context_item() {
        ui.text("context-item popup");
        ui.menu_item("action a");
        ui.menu_item("action b");
    }
}

// ---------------------------------------------------------------- tooltips

fn tab_tooltips(ui: &Ui, _s: &mut State) {
    ui.button("hover for tooltip");
    if ui.is_item_hovered() {
        ui.tooltip_text("simple tooltip text");
    }

    ui.button("hover for rich tooltip");
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            ui.text("rich");
            ui.separator();
            ui.text_colored([1.0, 0.5, 0.5, 1.0], "tooltip");
        });
    }

    ui.button("begin_tooltip");
    if ui.is_item_hovered() {
        if let Some(_t) = ui.begin_tooltip() {
            ui.text("token-based tooltip");
        }
    }
}

// ---------------------------------------------------------------- drag/drop

fn tab_dragdrop(ui: &Ui, s: &mut State) {
    ui.text("drag colored squares onto each other:");
    let mut swap: Option<(usize, u32)> = None;
    for i in 0..s.dnd_slots.len() {
        if i > 0 {
            ui.same_line();
        }
        let id = ui.push_id_int(i as i32);
        let color = s.dnd_slots[i];
        let _style = ui.push_style_color(StyleColor::Button, ImColor32::from(color).to_rgba_f32s());
        ui.button_with_size(format!("slot {i}"), [80.0, 80.0]);
        drop(_style);

        if let Some(t) = ui
            .drag_drop_source_config("color-payload")
            .flags(DragDropFlags::SOURCE_NO_PREVIEW_TOOLTIP)
            .begin_payload(color)
        {
            ui.text(format!("dragging slot {i}"));
            t.end();
        }

        if let Some(target) = ui.drag_drop_target() {
            if let Some(Ok(payload)) =
                target.accept_payload::<u32, _>("color-payload", DragDropFlags::empty())
            {
                swap = Some((i, payload.data));
            }
            target.pop();
        }
        id.pop();
    }
    if let Some((idx, color)) = swap {
        s.dnd_slots[idx] = color;
    }
}

// ---------------------------------------------------------------- layout

fn tab_layout(ui: &Ui, _s: &mut State) {
    ui.text("group:");
    ui.group(|| {
        ui.button("a");
        ui.button("b");
        ui.button("c");
    });
    ui.same_line();
    ui.text("←group acts as one item");

    ui.separator();
    ui.text("same_line / spacing / new_line / dummy:");
    ui.text("x");
    ui.same_line();
    ui.text("y");
    ui.same_line_with_spacing(0.0, 40.0);
    ui.text("z");
    ui.spacing();
    ui.dummy([10.0, 20.0]);
    ui.new_line();

    ui.separator();
    ui.text("indent / unindent:");
    ui.indent();
    ui.text("indented once");
    ui.indent_by(20.0);
    ui.text("indented twice");
    ui.unindent_by(20.0);
    ui.unindent();

    ui.separator();
    ui.text("child window:");
    if let Some(_c) = ui
        .child_window("child")
        .size([0.0, 80.0])
        .child_flags(ChildFlags::BORDERS)
        .horizontal_scrollbar(true)
        .begin()
    {
        for i in 0..30 {
            ui.text(format!("child line {i}"));
        }
    }

    ui.separator();
    ui.text("legacy columns:");
    ui.columns(3, "legacy-cols", true);
    for i in 0..6 {
        ui.text(format!("col cell {i}"));
        ui.next_column();
    }
    ui.columns(1, "end-cols", false);
}

// ---------------------------------------------------------------- stacks

fn tab_stacks(ui: &Ui, s: &mut State) {
    let _alpha = ui.push_style_var(StyleVar::Alpha(s.style_alpha));
    let _padding = ui.push_style_var(StyleVar::FramePadding([8.0, 4.0]));
    let _color = ui.push_style_color(StyleColor::Button, [0.2, 0.4, 0.8, 1.0]);
    ui.button("styled button");
    drop(_color);
    drop(_padding);
    drop(_alpha);

    ui.slider("style alpha", 0.2_f32, 1.0_f32, &mut s.style_alpha);

    ui.separator();
    ui.text("id stack:");
    for i in 0..3 {
        let _id = ui.push_id_int(i);
        if ui.button("same label") {
            log::info!("clicked button {i}");
        }
        ui.same_line();
    }
    ui.new_line();
    let _idstr = ui.push_id("string-id");
    ui.button("scoped");
    drop(_idstr);

    ui.separator();
    ui.text("item width stack:");
    let _w = ui.push_item_width(80.0);
    ui.input_float("narrow", &mut s.float_val).build();
    drop(_w);
    ui.set_next_item_width(200.0);
    ui.input_float("set_next_item_width", &mut s.float_val).build();

    ui.separator();
    let _wrap = ui.push_text_wrap_pos_with_pos(ui.cursor_pos()[0] + 200.0);
    ui.text(
        "text with explicit wrap position pushed onto the stack — this should wrap at 200px from the cursor.",
    );
    drop(_wrap);
}

// ---------------------------------------------------------------- plots

fn tab_plots(ui: &Ui, _s: &mut State) {
    let line: [f32; 64] =
        std::array::from_fn(|i| ((i as f32) * 0.2).sin() * 0.5 + 0.5);
    ui.plot_lines("sine", &line)
        .graph_size([0.0, 80.0])
        .scale_min(0.0)
        .scale_max(1.0)
        .overlay_text("sine wave")
        .build();

    let hist: [f32; 16] = std::array::from_fn(|i| (i as f32) * 0.06 + 0.1);
    ui.plot_histogram("hist", &hist)
        .graph_size([0.0, 80.0])
        .build();
}

// ---------------------------------------------------------------- draw list

fn tab_drawlist(ui: &Ui) {
    ui.text("foreground & window draw lists:");
    let p = ui.cursor_screen_pos();
    let dl = ui.get_window_draw_list();
    dl.add_line([p[0], p[1] + 10.0], [p[0] + 200.0, p[1] + 10.0], [1.0, 1.0, 0.0, 1.0])
        .thickness(2.0)
        .build();
    dl.add_rect(
        [p[0], p[1] + 20.0],
        [p[0] + 60.0, p[1] + 60.0],
        [0.2, 0.8, 0.2, 1.0],
    )
    .rounding(6.0)
    .thickness(1.0)
    .build();
    dl.add_rect(
        [p[0] + 70.0, p[1] + 20.0],
        [p[0] + 130.0, p[1] + 60.0],
        [0.8, 0.2, 0.2, 1.0],
    )
    .filled(true)
    .build();
    dl.add_circle([p[0] + 170.0, p[1] + 40.0], 18.0, [0.4, 0.6, 1.0, 1.0])
        .num_segments(24)
        .filled(true)
        .build();
    dl.add_triangle(
        [p[0] + 220.0, p[1] + 60.0],
        [p[0] + 260.0, p[1] + 60.0],
        [p[0] + 240.0, p[1] + 20.0],
        [1.0, 0.5, 0.0, 1.0],
    )
    .filled(true)
    .build();
    dl.add_bezier_curve(
        [p[0] + 280.0, p[1] + 60.0],
        [p[0] + 300.0, p[1] + 0.0],
        [p[0] + 340.0, p[1] + 80.0],
        [p[0] + 380.0, p[1] + 20.0],
        [0.9, 0.9, 0.9, 1.0],
    )
    .thickness(2.0)
    .num_segments(40)
    .build();
    dl.add_text([p[0], p[1] + 70.0], [1.0, 1.0, 1.0, 1.0], "draw list text");

    // reserve vertical space so following widgets don't overlap
    ui.dummy([0.0, 100.0]);

    // channels split/merge
    let p2 = ui.cursor_screen_pos();
    dl.channels_split(2, |split| {
        split.set_current(1);
        dl.add_rect(
            [p2[0], p2[1]],
            [p2[0] + 100.0, p2[1] + 30.0],
            [0.1, 0.4, 0.7, 1.0],
        )
        .filled(true)
        .build();
        split.set_current(0);
        dl.add_text([p2[0] + 6.0, p2[1] + 6.0], [1.0; 4], "on top via channels");
    });
    ui.dummy([0.0, 40.0]);
}

// ---------------------------------------------------------------- io / style (read-only)

fn tab_io_style(ui: &Ui) {
    let io = ui.io();
    ui.text(format!("display_size = {:?}", io.display_size));
    ui.text(format!("framerate = {:.1} fps", io.framerate));
    ui.text(format!("delta_time = {:.4} s", io.delta_time));
    ui.text(format!("mouse_pos = {:?}", io.mouse_pos));
    ui.text(format!(
        "mouse_down = [{}, {}, {}]",
        io.mouse_down[0], io.mouse_down[1], io.mouse_down[2]
    ));
    ui.text(format!("want_capture_mouse = {}", io.want_capture_mouse));
    ui.text(format!("want_capture_keyboard = {}", io.want_capture_keyboard));

    ui.separator();
    let style = ui.clone_style();
    ui.text(format!("alpha = {}", style.alpha));
    ui.text(format!("frame_rounding = {}", style.frame_rounding));
    ui.text(format!("window_padding = {:?}", style.window_padding));
    ui.text("(read-only — host owns the style)");
}

// ---------------------------------------------------------------- clipboard

fn tab_clipboard(ui: &Ui, s: &mut State) {
    ui.input_text("clip text", &mut s.text_buf).build();
    if ui.button("copy → clipboard") {
        ui.set_clipboard_text(&s.text_buf);
    }
    ui.same_line();
    if ui.button("paste ← clipboard") {
        if let Some(text) = ui.clipboard_text() {
            s.text_buf = text;
        }
    }
}
