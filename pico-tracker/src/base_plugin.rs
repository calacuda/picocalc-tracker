use crate::hal::{
    self, Sio,
    clocks::{Clock, init_clocks_and_plls},
    gpio::{FunctionI2C, Pin, PullUp},
    pac,
    powman::Powman,
    watchdog::Watchdog,
};
use bevy::prelude::*;
use display_interface_spi::SPIInterface;
use embedded_hal::spi::MODE_3;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{SdCard, VolumeManager};
use fugit::RateExtU32;
use pico_tracker_types::ron;
use pico_tracker_types::{FromHost, FromTracker};
use picocalc_bevy::{
    Display, DummyTimesource, FileSystemStruct, Keeb, KeyPresses, LoggingEnv, PicoTimer,
    XTAL_FREQ_HZ, clear_display, get_key_report,
    screen::{Command, Commands, ILI9486, color::PixelFormat, io::shim::OutputOnlyIoPin},
    start_timer, tick_timer,
};
use usb_device::{
    bus::UsbBusAllocator,
    device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid},
};
use usbd_midi::{
    CableNumber, UsbMidiClass,
    message::{Channel, Message, Note, Velocity},
};
use usbd_serial::SerialPort;

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        let mut pac = pac::Peripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        let clocks = init_clocks_and_plls(
            XTAL_FREQ_HZ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        // SETUP SCREEN
        // Pin<Gpio11, FunctionSpi, PullDown>,
        // Pin<Gpio12, FunctionSpi, PullDown>,
        // Pin<Gpio10, FunctionSpi, PullDown>,

        // These are implicitly used by the spi driver if they are in the correct mode
        // These are implicitly used by the spi driver if they are in the correct mode
        let dc = pins.gpio14.into_push_pull_output();
        let cs = pins.gpio13.into_push_pull_output();
        let mut rst = OutputOnlyIoPin::new(pins.gpio15.into_push_pull_output());
        let spi_mosi = pins.gpio11.into_function::<hal::gpio::FunctionSpi>();
        let spi_miso = pins.gpio12.into_function::<hal::gpio::FunctionSpi>();
        // #define RST_PIN 15
        let spi_sclk = pins.gpio10.into_function::<hal::gpio::FunctionSpi>();
        let spi_bus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI1, (spi_mosi, spi_miso, spi_sclk));

        // Exchange the uninitialised SPI driver for an initialised one
        let spi = spi_bus.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            200_000_000u32.Hz(),
            MODE_3,
        );

        let display_spi = SPIInterface::new(spi, dc, cs);

        let mut lcd_driver =
            ILI9486::new(&mut timer, PixelFormat::Rgb565, display_spi, &mut rst).unwrap();

        // reset
        lcd_driver.write_command(Command::Nop, &[]).unwrap();
        lcd_driver.write_command(Command::SleepOut, &[]).unwrap();

        lcd_driver
            .write_command(Command::DisplayInversionOn, &[])
            .unwrap();

        // MADCTL settings
        lcd_driver
            .write_command(Command::MemoryAccessControl, &[0b01001000])
            .unwrap();

        lcd_driver.clear_screen().unwrap();

        // turn on display
        lcd_driver
            .write_command(Command::NormalDisplayMode, &[])
            .unwrap();
        lcd_driver.write_command(Command::DisplayOn, &[]).unwrap();
        lcd_driver.write_command(Command::IdleModeOff, &[]).unwrap();
        lcd_driver
            .write_command(Command::TearingEffectLineOn, &[])
            .unwrap();

        // SETUP KEEB
        let keeb_addr = 0x1f;
        let i2c_freq = 200_000.Hz();
        let sda_pin: Pin<_, FunctionI2C, PullUp> = pins.gpio6.reconfigure();
        let scl_pin: Pin<_, FunctionI2C, PullUp> = pins.gpio7.reconfigure();

        // Create the I²C drive, using the two pre-configured pins. This will fail
        // at compile time if the pins are in the wrong mode, or if this I²C
        // peripheral isn't available on these pins!
        let i2c = hal::I2C::i2c1(
            pac.I2C1,
            sda_pin,
            scl_pin,
            i2c_freq,
            &mut pac.RESETS,
            &clocks.system_clock,
        );

        // TIMER setup
        let powman = Powman::new(pac.POWMAN, None);
        let pico_timer = PicoTimer::new(powman);

        // SD card osetup
        let cs = pins.gpio17.into_push_pull_output();
        let spi_mosi = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
        let spi_miso = pins.gpio16.into_function::<hal::gpio::FunctionSpi>();
        let spi_sclk = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
        let spi_bus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

        let spi = spi_bus.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            1_000_000.Hz(), // card initialization happens at low baud rate
            embedded_hal::spi::MODE_0,
        );
        let spi = ExclusiveDevice::new(spi, cs, timer).unwrap();
        let sdcard = SdCard::new(spi, timer);
        let volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());
        let fs = FileSystemStruct(volume_mgr);

        // start multi core sf2 player
        // let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
        // let cores = mc.cores();
        // let core1 = &mut cores[1];
        // let _test = core1.spawn(CORE1_STACK.take().unwrap(), move || core1_task(sys_freq));

        app.set_runner(move |mut app| {
            // usb logging
            let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
                pac.USB,
                pac.USB_DPRAM,
                clocks.usb_clock,
                true,
                &mut pac.RESETS,
            ));
            let mut serial = SerialPort::new(&usb_bus);
            //
            // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
            //     .strings(&[StringDescriptors::default()
            //         .manufacturer("calacuda")
            //         .product("Ferris")
            //         .serial_number("TEST")])
            //     .unwrap()
            //     .device_class(2) // 2 for the CDC, from: https://www.usb.org/defined-class-codes
            //     .build();
            // let _ = usb_dev.poll(&mut [&mut serial]);
            //
            // serial.write(b"starting bevy\n").unwrap();

            // Create a MIDI class with 1 input and 1 output jack.
            let mut midi = UsbMidiClass::new(&usb_bus, 1, 5).unwrap();

            let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x5e4))
                .device_class(0)
                .device_sub_class(0)
                .strings(&[StringDescriptors::default()
                    .manufacturer("calacuda")
                    .product("MidiOxide")
                    .serial_number("12345678")])
                .unwrap()
                .build();

            loop {
                // let _ = usb_dev.poll(&mut [&mut serial]);
                let _ = usb_dev.poll(&mut [&mut midi, &mut serial]);
                // let _ = usb_dev.poll(&mut [&mut midi]);

                let ser_write = |serial: &mut SerialPort<_>, message: String| {
                    let res = serial.write(&message.into_bytes());
                    _ = serial.write(&['\n' as u8, '\r' as u8]);

                    res
                };

                let mut buf = [0u8; 512];

                if let Ok(count) = serial.read(&mut buf[..]) {
                    let raw_ron_msg: String = buf[..count]
                        .into_iter()
                        .filter_map(|byte| {
                            let byte = *byte;
                            // for the terminating Null byte.
                            if byte != 0 { Some(byte as char) } else { None }
                        })
                        .collect();

                    if let Ok(message) = ron::from_str(&raw_ron_msg) {
                        let world = app.world_mut();
                        if let Some(ref mut events) = world.get_resource_mut::<Events<FromHost>>() {
                            events.send(message);
                        }
                    } else {
                        let _ = ron::to_string(&FromTracker::Log {
                            message: format!("failed to read to ron: {raw_ron_msg}"),
                        })
                        .map(|msg| ser_write(&mut serial, msg));
                    }
                };

                app.update();

                {
                    let world = app.world_mut();
                    if let Some(ref mut events) = world.get_resource_mut::<Events<LoggingEnv>>() {
                        for event in events.iter_current_update_events() {
                            // let _ = serial.write(&event.msg.clone().into_bytes());
                            // let _ = serial.write(&['\n' as u8, '\r' as u8]);

                            let _ = ron::to_string(&FromTracker::Log {
                                message: event.msg.clone(),
                            })
                            .map(|msg| ser_write(&mut serial, msg));
                        }

                        events.update();
                    }
                }

                {
                    let world = app.world_mut();
                    if let Some(ref mut events) = world.get_resource_mut::<Events<FromTracker>>() {
                        for event in events.iter_current_update_events() {
                            // let _ = serial.write(&[0]);
                            // let _ = serial.write(&event.msg.clone().into_bytes());
                            // let _ = serial.write(&['\n' as u8, '\r' as u8]);
                            let _ = ron::to_string(event).map(|msg| ser_write(&mut serial, msg));
                        }

                        events.update();
                    }
                }

                {
                    let world = app.world_mut();
                    if let Some(ref mut events) = world.get_resource_mut::<Events<MidiEnv>>() {
                        for event in events.iter_current_update_events() {
                            let packet = match *event {
                                MidiEnv::On { note, vel } => Message::NoteOn(
                                    Channel::Channel1,
                                    Note::try_from(note).unwrap(),
                                    Velocity::try_from(vel).unwrap(),
                                ),
                                MidiEnv::Off { note } => Message::NoteOff(
                                    Channel::Channel1,
                                    Note::try_from(note).unwrap(),
                                    Velocity::try_from(120).unwrap(),
                                ),
                            };

                            let _ = midi.send_packet(packet.into_packet(CableNumber::Cable0));
                        }

                        events.update();
                    }
                }

                if let Some(exit) = app.should_exit() {
                    return exit;
                }
            }
        })
        .add_event::<LoggingEnv>()
        .add_event::<MidiOutEnv>()
        .add_event::<MidiEnv>()
        .add_event::<FromHost>()
        .add_event::<FromTracker>()
        .insert_non_send_resource(Keeb {
            i2c,
            adr: keeb_addr,
            speed: i2c_freq,
        })
        .insert_non_send_resource(Display { output: lcd_driver })
        .insert_non_send_resource(timer)
        .insert_non_send_resource(pico_timer)
        .insert_non_send_resource(rst)
        // TODO: make a non_send_resource to hold the unused pins which are exposed on the side of
        // the device, the remaining I2C interface, the unused UART, & PIO state machines.
        .insert_non_send_resource(fs)
        .insert_resource(KeyPresses::default())
        // .insert_resource(DoubleFrameBuffer::new(320, 320))
        // .insert_non_send_resource(DoubleFrameBuffer::new(lcd_driver, 320, 320))
        .add_systems(Startup, (start_timer, tick_timer, clear_display))
        .add_systems(Update, get_key_report)
        // .add_systems(Update, usb_poll)
        .add_systems(PostUpdate, tick_timer);
    }
}

#[derive(Event, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct MidiOutEnv {
    pub msg: String,
}

#[derive(Event, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum MidiEnv {
    On { note: u8, vel: u8 },
    Off { note: u8 },
}
