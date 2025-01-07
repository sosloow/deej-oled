use embassy_time::{Duration, Ticker};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};
use heapless::String;

use crate::screen;

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    let mut ticker = Ticker::every(Duration::from_millis(15));

    let mut counter: u32 = 0;

    let style = MonoTextStyle::new(
        &embedded_graphics::mono_font::iso_8859_1::FONT_10X20,
        Gray4::WHITE,
    );

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        let mut buffer: String<32> = String::try_from("Hello from\nthe SSD1322\n").unwrap();
        let c: String<8> = counter.try_into().unwrap();
        buffer.push_str(c.as_str()).unwrap();
        Text::with_alignment(&buffer, Point::new(128, 12), style, Alignment::Center)
            .draw(frame)
            .unwrap();

        screen::READY_FRAME.signal(frame);
        counter += 1;

        ticker.next().await;
    }
}
