//! A simple demonstration of how to construct and use Canvasses by splitting up the window.

#[macro_use] extern crate conrod;
mod support;


fn main() {
    feature::main();
}

mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use support;

    pub fn main() {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;

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

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();

        'main: loop {
            
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
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            // Instantiate all widgets in the GUI.
            set_widgets(ui.set_widgets(), ids);

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
    fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids) {
        use conrod::{color, widget, Colorable, Borderable, Positionable, Sizeable, Widget};
        
        let main_color = color::rgb(0.2,0.2,0.3);
        let other_color = color::rgb(0.1,0.1,0.2);

        
        // Construct our main `Canvas` tree.
        widget::Canvas::new().flow_down(&[
            (ids.header, widget::Canvas::new().color(main_color).length(100.0)),
            (ids.body,  widget::Canvas::new().color(color::ORANGE).scroll_kids_vertically()),
        ]).set(ids.master, ui);

        // A scrollbar for the `FOOTER` canvas.
        widget::Scrollbar::y_axis(ids.body).auto_hide(false).set(ids.footer_scrollbar, ui);

        widget::Text::new("Time tracker")
            .color(color::LIGHT_ORANGE)
            .font_size(48)
            .middle_of(ids.header)
            .set(ids.title, ui);

        // Here we make some canvas `Tabs` in the middle column.
        widget::Tabs::new(&[(ids.tab_timers, "Timers"), (ids.tab_statistics, "Statistics")])
            .wh_of(ids.body)
            .color(other_color)
            .border(0.)
            .label_color(color::WHITE)
            .middle_of(ids.body)
            .set(ids.tabs, ui);


        let timer = &mut Timer::new(ui.widget_id_generator());
        widget::Canvas::new()
            .wh_of(ids.tab_timers)
            .color(main_color)
            .middle_of(ids.tab_timers)
            .flow_right(&[
                (timer.toggle, widget::Canvas::new()),
                (timer.clock, widget::Canvas::new()),
            ]).set(timer.master,ui);

        widget::Toggle::new(false)
            .wh_of(timer.toggle)
            .color(other_color)
            .middle_of(timer.toggle)
            .set(timer.toggle_button,ui);
        //duration
        //deltatime
        //fn text (text: widget::Text) -> widget::Text { text.color(color::WHITE).font_size(36) }


    }


    // Generate a unique `WidgetId` for each widget.
    widget_ids! {
        struct Ids {
            master,
            header,
            body,

            footer_scrollbar,

            tabs,
            tab_timers,
            tab_statistics,

            title,
            subtitle,

        }
    }
    widget_ids! {
        struct Timer {
            master,
            toggle,
            clock,

            toggle_button,
            current,

        }
    }
}