extern crate sfml;

use sfml as sf;

fn main() {
    let settings = sf::window::ContextSettings {
        depth_bits: 24,
        stencil_bits: 8,
        antialiasing_level: 4,
        major_version: 3,
        minor_version: 0,
        attribute_flags: 1,
        srgb_capable: 0
    };

    let mut window = sf::graphics::RenderWindow::new(
        (512, 512),
        "sphere",
        sf::window::Style::TITLEBAR | sf::window::Style::CLOSE,
        &settings,
    );

    window.set_framerate_limit(60);
    while window.is_open() {
        while let Some(event) = window.poll_event() {

            match event {
                sfml::window::Event::Closed => window.close(),
                _ => {}
            }
        }

        window.display();
    }

    println!("Hello, world!");
}
