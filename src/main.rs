//! A simple demonstration of how to construct and use Canvasses by splitting up the window.

#[macro_use] extern crate conrod;
#[macro_use] extern crate serde_derive;

extern crate chrono;
extern crate serde;
mod support;

fn main() {
    feature::main();
}

mod feature {
    const FILENAME : &str = "timetracker.json";


    extern crate find_folder;

    use std::fs::File;
    use std::io::prelude::*;
    use std::time::Duration;
    use std::thread::sleep;

    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
  
   
    extern crate serde_json;

    use support;
    use chrono::prelude::*;
    use chrono;
    
    pub fn main() {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;
        const SLEEPTIME: Duration = Duration::from_millis(500);

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Timetracker")
            .with_dimensions(WIDTH, HEIGHT);
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Instantiate the generated list of widget identifiers.
        let ids = &mut Ids::new(ui.widget_id_generator());
        let mut ids_list = Vec::new();
        let mut curname = "Enter name".to_string();

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();

        let mut timerstates : Vec<support::TimerState> = match File::open(FILENAME) {
            Ok(mut a) => {
                let mut s = String::new();
                a.read_to_string(&mut s).expect("Failed to read config");
                serde_json::from_str(&s).expect("Failed convert to json")
            },
            Err(_e) => {
                Vec::new()
            }
        };
        'main: loop {
            sleep(SLEEPTIME);
            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::Closed |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => {
                            let mut f = File::create(FILENAME).unwrap();
                            f.write_all(serde_json::to_string(&timerstates)
                                .unwrap()
                                .as_bytes()).unwrap();
                            
                            break 'main
                        },
                        _ => (),
                    },
                    _ => (),
                }
            }

            // Instantiate all widgets in the GUI.
            set_widgets(ui.set_widgets(), ids, &mut ids_list, &mut timerstates, &mut curname);

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Draw the Ui.
    fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids, ids_list: &mut Vec<ListItem>, timerstates : &mut Vec<support::TimerState> ,text : &mut String) {
        use conrod::{color, widget, Colorable, Borderable, Positionable, Labelable, Sizeable, Widget};
        
        let main_color = color::rgb(0.2,0.2,0.3);
        let other_color = color::rgb(0.1,0.1,0.2);
        let green_color = color::rgb(0.45,1.,0.12);

        
        // Construct our main `Canvas` tree.
        widget::Canvas::new().flow_down(&[
            (ids.header, widget::Canvas::new().color(main_color).length(70.0)),
            (ids.body,  widget::Canvas::new().color(color::ORANGE).scroll_kids_vertically()),
        ]).set(ids.master, ui);

        // A scrollbar for the `FOOTER` canvas.
        widget::Scrollbar::y_axis(ids.body).auto_hide(false).set(ids.footer_scrollbar, ui);

        widget::Text::new("Time tracker")
            .color(color::LIGHT_ORANGE)
            .font_size(28)
            .mid_left_with_margin_on(ids.header,28.)
            .left_justify()
            .set(ids.title, ui);

        // Here we make some canvas `Tabs` in the middle column.
        widget::Tabs::new(&[(ids.tab_timers, "Timers")/*,(ids.tab_statistics, "Statistics")*/])
            .wh_of(ids.body)
            .color(other_color)
            .border(0.)
            .label_color(color::WHITE)
            .middle_of(ids.body)
            .set(ids.tabs, ui);

        while ids_list.len() < timerstates.len() {
            ids_list.push(ListItem::new(ui.widget_id_generator()));
        }

        let (mut items, _scrollbar) = widget::List::flow_down(timerstates.len())
            .item_size(50.0)
            .scrollbar_on_top()
            .middle_of(ids.tab_timers)
            .wh_of(ids.tab_timers)
            .set(ids.timer_list, ui);


        while let Some(item) = items.next(ui) {
            let i = item.i;
            let mut  label;

            let dummy = widget::Canvas::new().w_of(ids.timer_list);
            item.set(dummy , ui);

            widget::Canvas::new()
                .wh_of(item.widget_id)
                .middle_of(item.widget_id)
                .set(ids_list[i].master, ui);
            
            //Make the label for the toggle button
            if timerstates[i].active {
                let zero : u32 = 0;
                let timesince : DateTime<Utc> = chrono::MIN_DATE.and_hms(zero,zero,zero).checked_add_signed(duration_elapsed(timerstates[i].active_since)).unwrap();
                let delta = format_time(timesince);
                label = format!("{}", delta);
            }
            else {
                label = format!("{}",format_time(timerstates[i].total));
            }
            for b in  widget::Toggle::new(timerstates[i].active)
                .h_of(ids_list[i].master)
                .padded_w_of(ids_list[i].master,25.)
                .label(&label)
                .label_color(if timerstates[i].active {color::BLACK} else {color::LIGHT_ORANGE})
                .mid_left_of(ids_list[i].master)
                .color(if timerstates[i].active  {green_color}else {other_color})
                .set(ids_list[i].toggle, ui) {
                if b {
                    timerstates[i].active_since = Utc::now();
                }
                else {
                    timerstates[i].total = timerstates[i].total.checked_add_signed(duration_elapsed(timerstates[i].active_since)).unwrap();
                }
                timerstates[i].active = b;
            }
            
            widget::Text::new(timerstates[i].name.as_str())
                .color(if timerstates[i].active {color::BLACK} else {color::LIGHT_ORANGE})
                .font_size(28)
                .bottom_left_with_margin_on(ids_list[i].toggle,14.)
                .left_justify()
                .set(ids_list[i].name, ui);

            for _press in widget::Button::new()
                .h_of(ids_list[i].master)
                .w(50.)
                .label("-")
                .mid_right_of(ids_list[i].master)
            .set(ids_list[i].remove, ui){
                timerstates.remove(i);
                ids_list.remove(i);
                return;
            }




            
            
        }

        for edit in widget::TextBox::new(text)
            .color(color::WHITE)
            .h(50.)
            .padded_w_of(ids.tab_timers, 25.0)
            .bottom_left_of(ids.tab_timers)
            .center_justify()
            .set(ids.add_name, ui)
        {
            use conrod::widget::text_box::Event::{Update,Enter};
            match edit {
                Update(txt) => {
                    *text = txt;
                },
                Enter => {
                    timerstates.push(support::TimerState::new(text.clone()));
                },
            }
            
        }

        for _press in widget::Button::new()
            .h(50.)
            .w(50.)
            .label("+")
            .bottom_right_of(ids.tab_timers)
            .set(ids.plus_button, ui){
                timerstates.push(support::TimerState::new(text.clone()));
            }
        

    }
    fn format_time(t : chrono::DateTime<Utc>) -> String {
        let dur = t.signed_duration_since(chrono::MIN_DATE.and_hms(0u32,0u32,0u32));
        let ret = format!(
            "{:02}:{:02}:{:02}",
            dur.num_hours(),
            dur.num_minutes()%60,
            dur.num_seconds()%60
        );
        ret
    }
    fn duration_elapsed(t : chrono::DateTime<Utc>) -> chrono::Duration {
        chrono::offset::Utc::now().signed_duration_since(t)
    }


    // Generate a unique `WidgetId` for each widget.
    widget_ids! {
        struct Ids {
            master,
            header,
            body,
            
            timer_list,
            plus_button,
            add_name,

            footer_scrollbar,

            tabs,
            tab_timers,
            tab_statistics,

            title,
            subtitle,

        }
    }
    widget_ids! {
        struct ListItem {
            master,

            toggle,
            remove,

            name,
            time,
            session,
        }
    }
}