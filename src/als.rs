#![no_std]
#![no_main]

use cortex_m_rt::entry;
use core::{fmt::Write};
use panic_halt as _;
use stm32f4xx_hal::{
    i2c::I2c, 
    pac, 
    prelude::*, 
    rcc::Config,
    serial::{config::Config as SerialConfig, Serial},
};

use vd6283::{get_lux_cct, Channel, Vd6283};

/// STM32F411CEU6 的 VD6283 应用配置。
const I2C_BUS_HZ: u32 = 100_000;
const ALS_CHANNELS: u8 = 0x3f;
const EXPOSURE_US: u32 = 100_000;

/// 初始化 I2C1 和 VD6283，然后周期性读取 Lux/CCT 并通过 RTT 输出。
#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // 使用 HSI 16 MHz 作为 PLL 输入，系统主频设置为 100 MHz。
    let mut rcc = dp
        .RCC
        .freeze(Config::hsi().sysclk(100.MHz()));

    let gpio_a = dp.GPIOA.split(&mut rcc);
    let gpio_b = dp.GPIOB.split(&mut rcc);

    let mut serial1 = Serial::new(
        dp.USART1,
        (gpio_a.pa9, gpio_a.pa10),
        SerialConfig::default().baudrate(115_200.bps()),
        &mut rcc,
    )
    .unwrap()
    .with_u8_data();

    let i2c = I2c::new(
        dp.I2C1,
        (gpio_b.pb6, gpio_b.pb7),
        I2C_BUS_HZ.Hz(),
        &mut rcc,
    );
    let mut delay = cp.SYST.delay(&rcc.clocks);
    let mut sensor = Vd6283::new(i2c);

    if let Err(error) = sensor.init() {
        writeln!(serial1, "VD6283 init failed: {:?}", error).ok();
        loop {
            delay.delay_ms(1_000_u32);
        }
    }

    writeln!(serial1, 
        "VD6283 detected: id=0x{:02x}, revision=0x{:02x}",
        sensor.device_id(),
        sensor.revision_id()
    ).ok();

    if let Err(error) = sensor.set_exposure_time(EXPOSURE_US) {
        writeln!(serial1, "set exposure failed: {:?}", error).ok();
    }
    for channel in [
        Channel::Ch1,
        Channel::Ch2,
        Channel::Ch3,
        Channel::Ch4,
        Channel::Ch5,
        Channel::Ch6,
    ] {
        if let Err(error) = sensor.set_gain(channel, 0x0100) {
            writeln!(serial1, "set gain failed: {:?}", error).ok();
        }
    }

    loop {
        if let Err(error) = sensor.start_single_shot(ALS_CHANNELS) {
            writeln!(serial1, "start ALS failed: {:?}", error).ok();
            delay.delay_ms(200_u32);
            continue;
        }

        loop {
            match sensor.read_als(ALS_CHANNELS) {
                Ok(Some(als)) => {
                    let result = get_lux_cct(&als, sensor.exposure_time_us());
                    writeln!(serial1, 
                        "Lux={:.2}, CCT={:.2} K, raw=[{}, {}, {}, {}, {}, {}]",
                        result.lux,
                        result.cct,
                        als.count_value_raw[0],
                        als.count_value_raw[1],
                        als.count_value_raw[2],
                        als.count_value_raw[3],
                        als.count_value_raw[4],
                        als.count_value_raw[5]
                    ).ok();
                    break;
                }
                Ok(None) => delay.delay_ms(1_u32),
                Err(error) => {
                    writeln!(serial1, "read ALS failed: {:?}", error).ok();
                    break;
                }
            }
        }

        if let Err(error) = sensor.stop_als() {
            writeln!(serial1, "stop ALS failed: {:?}", error).ok();
        }
        delay.delay_ms(200_u32);
    }
}
