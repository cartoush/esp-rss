use anyhow::Result;
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    iterator::pixel,
    mono_font::{iso_8859_16::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::{Dimensions, DrawTarget, Point, RgbColor, Size},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    text::Text,
    Drawable, Pixel,
};
use embedded_svc::http::client::Client;
use esp_idf_sys::{self as _};
// use lvgl::{DrawBuffer, style::Style, Color, Part, Align, widgets::{Label, Arc}, Widget};
use serde::de::IntoDeserializer;

use std::thread::sleep;
use std::time::Duration;
use std::{cell::RefCell, error::Error, ffi::CString, io::Write, panic, rc::Rc, time::Instant};

use esp_idf_hal::{delay, gpio::AnyIOPin, gpio::PinDriver, i2c, prelude::*, rmt::config, spi};
use esp_idf_svc::{http::client::*, timer::EspTimerService};

use slint;

use mipidsi::*;

use log::info;

mod freshrss;
mod serde_rss;
mod wifi;

const SCR_WIDTH: u16 = 320;
const SCR_HEIGHT: u16 = 240;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_pwd: &'static str,
    #[default("")]
    rss_domain: &'static str,
    #[default("")]
    rss_username: &'static str,
    #[default("")]
    rss_password: &'static str,
    #[default("")]
    host_ip: &'static str,
    #[default(8529)]
    host_port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_sys::link_patches();

    unsafe {
        esp_idf_sys::nvs_flash_init();

        // // Disable IDLE task WatchDogTask on this CPU.
        // esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandleForCPU(
        //     esp_idf_hal::cpu::core() as u32,
        // ));

        // // Enable WatchDogTask on the main (=this) task.
        // esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetCurrentTaskHandle());
        // let ret = esp_idf_sys::esp_task_wdt_status(esp_idf_sys::xTaskGetCurrentTaskHandle());
        // info!("ret : {:?}", ret);
        // esp_idf_sys::esp_task_wdt_delete(esp_idf_sys::xTaskGetIdleTaskHandle());
        // let ret = esp_idf_sys::esp_task_wdt_status(esp_idf_sys::xTaskGetIdleTaskHandle());
        // info!("ret : {:?}", ret);
    };

    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    let my_spi = spi::SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio14,
        peripherals.pins.gpio13,
        Some(peripherals.pins.gpio12),
        Option::<AnyIOPin>::None,
        &spi::SpiDriverConfig::new(),
        &spi::SpiConfig::new(),
    )?;

    // create a DisplayInterface from SPI and DC pin, with no manual CS control
    let di = SPIInterface::new(
        my_spi,
        PinDriver::output(peripherals.pins.gpio16)?,
        PinDriver::output(peripherals.pins.gpio15)?,
    );

    let mut display = Builder::ili9341_rgb565(di)
        .with_display_size(SCR_WIDTH, SCR_HEIGHT)
        .with_orientation(Orientation::Landscape(false))
        .init(
            &mut delay::Ets,
            Some(PinDriver::output(peripherals.pins.gpio17)?),
        )
        .map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?;

    Rectangle::new(
        display.bounding_box().top_left,
        Size::new(
            display.bounding_box().size.height,
            display.bounding_box().size.width,
        ),
    )
    .into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::RED)
            .stroke_color(Rgb565::RED)
            .stroke_width(1)
            .build(),
    )
    .draw(&mut display)
    .map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?;

    let style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::BLACK);
    Text::new("Hello Rust le monde!", Point::new(20, 30), style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?;

    let mut my_i2c = i2c::I2cDriver::new(
        peripherals.i2c1,
        peripherals.pins.gpio22,
        peripherals.pins.gpio21,
        &i2c::I2cConfig::default(),
    )?;
    let mut pin5 = PinDriver::input(peripherals.pins.gpio5)?;
    PinDriver::enable_interrupt(&mut pin5)?;
    let mut touch = ft6x06::Ft6X06::new(&my_i2c, 0x38, pin5)?;

    // info!("Waiting for wifi to connect");
    // sleep(Duration::from_millis(100));
    // // Connect to the Wi-Fi network
    // let _wifi = wifi::wifi(
    //     app_config.wifi_ssid,
    //     app_config.wifi_pwd,
    //     peripherals.modem,
    //     sysloop,
    // )?;
    // while _wifi.is_connected()? == false {sleep(Duration::from_millis(100));}
    // info!("Wifi is connected");

    // let conn = EspHttpConnection::new(&Configuration {
    //     use_global_ca_store: true,
    //     crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
    //     ..Default::default()
    // })?;
    // let mut cli = Client::wrap(conn);

    // let auth_string = freshrss::freshrss_connect(
    //     &mut cli,
    //     app_config.rss_domain,
    //     app_config.rss_username,
    //     app_config.rss_password,
    // )?;

    // let str_articles =
    //     freshrss::freshrss_get_articles(&mut cli, &auth_string, app_config.rss_domain)?;
    // let articles: serde_rss::RssReadingList = serde_json::from_str(str_articles.as_str())?;

    // info!("articles : {:?}", articles);

    slint::platform::set_platform(Box::new(EspBackend::default()))
        .expect("backend already initialized");
    let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );
    window.set_size(slint::PhysicalSize::new(SCR_WIDTH as _, SCR_HEIGHT as _));
    let mut line_buffer = [slint::platform::software_renderer::Rgb565Pixel(0); SCR_WIDTH as usize];

    HelloWorld::new().unwrap().run().unwrap();

    // sleep(Duration::from_secs(1));
    // let host_address = format!("{}:{}", app_config.host_ip, app_config.host_port);
    // info!("host_address : {:?}", host_address);
    // let mut tcp = std::net::TcpStream::connect(host_address.as_str())?;
    // let to_send = "coucou".as_bytes();
    // tcp.write_all(to_send)?;

    // let buffer = DrawBuffer::<{ (SCR_WIDTH as usize * SCR_HEIGHT as usize) as usize }>::default();
    // let lvgl_display = lvgl::Display::register(buffer, SCR_WIDTH as u32, SCR_HEIGHT as u32, |refresh| {
    //     let pixels = refresh.colors.map(|pix| Rgb565::new(pix.r(), pix.g(), pix.b()));
    //     display.set_pixels(0, 0, SCR_WIDTH, SCR_HEIGHT, pixels).map_err(
    //         |e| anyhow::anyhow!("Display error : {:?}", e)).expect("Error happened");
    // }).map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?;

    // let mut screen = lvgl_display.get_scr_act().map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?;

    loop {
        let is_touched = touch.td_status(&mut my_i2c).expect("is_touched fail");
        if is_touched > 0 && is_touched != 255 {
            // returns (y, x)
            let pos = touch.get_coordinates(&mut my_i2c)?;
            info!(
                "is_touched : {:?} touch coordinates : {:?}",
                is_touched, pos
            );
            if pos.0 > SCR_HEIGHT || pos.1 > SCR_WIDTH {
                info!("pos fucked : {:?}", pos);
                continue;
            }
            Pixel(
                Point::new((SCR_WIDTH - pos.1).into(), pos.0.into()),
                Rgb565::GREEN,
            )
            .draw(&mut display)
            .map_err(|e| anyhow::anyhow!("Display error : {:?}", e))?
        }

        window.draw_if_needed(|renderer| {
            info!("drawing ");
            renderer.render_by_line(DisplayWrapper {
                display: &mut display,
                line_buffer: &mut line_buffer,
            });
        });
        sleep(Duration::from_millis(10));
    }
}

struct DisplayWrapper<'a, T> {
    display: &'a mut T,
    line_buffer: &'a mut [slint::platform::software_renderer::Rgb565Pixel],
}
impl<T: DrawTarget<Color = embedded_graphics::pixelcolor::Rgb565>>
    slint::platform::software_renderer::LineBufferProvider for DisplayWrapper<'_, T>
{
    type TargetPixel = slint::platform::software_renderer::Rgb565Pixel;
    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        // Render into the line
        render_fn(&mut self.line_buffer[range.clone()]);

        // Send the line to the screen using DrawTarget::fill_contiguous
        self.display
            .fill_contiguous(
                &Rectangle::new(
                    Point::new(range.start as _, line as _),
                    Size::new(range.len() as _, 1),
                ),
                self.line_buffer[range.clone()]
                    .iter()
                    .map(|p| RawU16::new(p.0).into()),
            )
            .map_err(drop)
            .unwrap();
    }
}

#[derive(Default)]
struct EspBackend {
    window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
}

impl slint::platform::Platform for EspBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        let timer = match EspTimerService::new() {
            Ok(it) => it,
            Err(_) => return core::time::Duration::default(),
        };
        timer.now()
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        info!("event loop called");
        Ok(())
    }
}

slint::slint! {
export component HelloWorld inherits Rectangle {
    width: 320px;
    height: 240px;
    background: #a16277;

    Text {
       y: parent.height / 2;
       x: parent.width / 2;
       text: "Hello, world";
       color: red;
    }
}
}
