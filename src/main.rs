#![deny(unsafe_code)]
#![no_std]
#![no_main]

mod fmt;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use ThermoSoft_rs::{BATCH_SIZE, SensorDataPacket, log_faults, max31856};

use defmt::info;

use embassy_executor::Spawner;
use embassy_net::{
    Ipv4Address, Ipv4Cidr, StackResources,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk,
    VoltageScale,
};
use embassy_stm32::rng::Rng;
use embassy_stm32::spi::{MODE_1, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, eth, peripherals, rng};
use embassy_time::Timer;

use heapless::Vec;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
async fn net_task(
    mut runner: embassy_net::Runner<'static, Ethernet<'static, ETH, GenericPhy>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    config.rcc.hsi = None;
    config.rcc.hsi48 = Some(Default::default()); // needed for RNG
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV5,
        mul: PllMul::MUL100,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV10), // 50 MHz for ETH
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV5,
        mul: PllMul::MUL80,
        divp: Some(PllDiv::DIV80),
        divq: Some(PllDiv::DIV5), // 80 MHz for FDCAN
        divr: Some(PllDiv::DIV2),
    });

    config.rcc.pll3 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV5,
        mul: PllMul::MUL48,
        divp: Some(PllDiv::DIV5),
        divq: Some(PllDiv::DIV5), // 48 MHz for USB
        divr: Some(PllDiv::DIV5),
    });

    config.rcc.apb3_pre = APBPrescaler::DIV2; // 125 MHz
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2; // 125 MHz
    config.rcc.apb2_pre = APBPrescaler::DIV2; // 125 MHz
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.voltage_scale = VoltageScale::Scale0;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    // Toggle ETH NRST (PA0) for reset
    let mut eth_nrst = Output::new(p.PA0, Level::Low, Speed::Low);
    Timer::after_millis(100).await;
    eth_nrst.set_high();
    Timer::after_millis(2000).await;

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0xE8, 0x80, 0x88, 0x59, 0x0D, 0x70];

    static PACKETS: StaticCell<PacketQueue<16, 16>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<16, 16>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PB12,
        p.PB15,
        p.PA5,
        GenericPhy::new_auto(),
        mac_addr,
    );

    // let config = embassy_net::Config::dhcpv4(Default::default());
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 88, 61), 24),
        dns_servers: Vec::<Ipv4Address, 3>::new(),
        gateway: Some(Ipv4Address::new(192, 168, 88, 1)),
        // gateway: None
    });

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner
        .spawn(net_task(runner))
        .expect("Network task failed to spawn.");

    // Wait for link to come up first
    info!("Waiting for link to come up...");
    loop {
        if stack.is_link_up() {
            info!("Link is UP!");
            break;
        }
        Timer::after_millis(100).await;
    }

    // Ensure network configuration is up
    stack.wait_config_up().await;

    info!("Network task initialized");
    info!("IP address: {:?}", stack.config_v4());
    info!("Link is up: {}", stack.is_link_up());

    let mut spi_config = embassy_stm32::spi::Config::default();
    spi_config.mode = MODE_1; // MAX31856 requires Mode 1 or Mode 3
    spi_config.frequency = Hertz(5_000_000);
    let spi = Spi::new_blocking(p.SPI1, p.PB3, p.PB5, p.PB4, spi_config);

    // Create shared SPI bus using RefCell for blocking SPI
    use core::cell::RefCell;
    use embedded_hal_bus::spi::RefCellDevice;

    let spi_bus = RefCell::new(spi);

    let cs1 = Output::new(p.PA15, Level::High, Speed::VeryHigh); // CS1
    let cs2 = Output::new(p.PC12, Level::High, Speed::VeryHigh); // CS2
    let cs3 = Output::new(p.PC14, Level::High, Speed::VeryHigh); // CS3
    let cs4 = Output::new(p.PC2, Level::High, Speed::VeryHigh); // CS4

    let mut nfault1 = Input::new(p.PA10, Pull::Down); // NFAULT1
    let mut nfault2 = Input::new(p.PC11, Pull::Down); // NFAULT2
    let mut nfault3 = Input::new(p.PC13, Pull::Down); // NFAULT3
    let mut nfault4 = Input::new(p.PC0, Pull::Down); // NFAULT4

    let mut ndrdy1 = Input::new(p.PA9, Pull::Up); // DRDY1
    let mut ndrdy2 = Input::new(p.PA8, Pull::Up); // DRDY2
    let mut ndrdy3 = Input::new(p.PC15, Pull::Up); // DRDY3
    let mut ndrdy4 = Input::new(p.PC3, Pull::Up); // DRDY4

    // Create SPI devices using RefCellDevice
    let mut spi_dev1 = RefCellDevice::new(&spi_bus, cs1, embassy_time::Delay).unwrap();
    let mut spi_dev2 = RefCellDevice::new(&spi_bus, cs2, embassy_time::Delay).unwrap();
    let mut spi_dev3 = RefCellDevice::new(&spi_bus, cs3, embassy_time::Delay).unwrap();
    let mut spi_dev4 = RefCellDevice::new(&spi_bus, cs4, embassy_time::Delay).unwrap();

    // Configure all sensors with verification
    info!("Configuring and verifying all sensors...");
    ThermoSoft_rs::configure_and_verify_max31856(&mut spi_dev1, 1)
        .expect("Failed to configure sensor 1");
    ThermoSoft_rs::configure_and_verify_max31856(&mut spi_dev2, 2)
        .expect("Failed to configure sensor 2");
    ThermoSoft_rs::configure_and_verify_max31856(&mut spi_dev3, 3)
        .expect("Failed to configure sensor 3");
    ThermoSoft_rs::configure_and_verify_max31856(&mut spi_dev4, 4)
        .expect("Failed to configure sensor 4");

    // UDP socket setup
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer = [0; 512];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buffer = [0; 512];

    let mut udp_socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    // Bind to any local port
    udp_socket.bind(0).unwrap();

    // Remote UDP destination (hardcoded)
    let remote_endpoint = (Ipv4Address::new(192, 168, 88, 100), 8000);
    info!("Will send UDP packets to {:?}", remote_endpoint);

    let mut packet = SensorDataPacket::new();
    let mut batch_index = 0usize;
    let mut packet_counter = 0u32;

    loop {
        // Read each sensor with fault checking
        let (tc1, faults1) =
            max31856::read_thermocouple_with_fault_check(&mut spi_dev1, &mut nfault1, &mut ndrdy1)
                .await;
        let (tc2, faults2) =
            max31856::read_thermocouple_with_fault_check(&mut spi_dev2, &mut nfault2, &mut ndrdy2)
                .await;
        let (tc3, faults3) =
            max31856::read_thermocouple_with_fault_check(&mut spi_dev3, &mut nfault3, &mut ndrdy3)
                .await;
        let (tc4, faults4) =
            max31856::read_thermocouple_with_fault_check(&mut spi_dev4, &mut nfault4, &mut ndrdy4)
                .await;

        // Log faults if present
        if let Some(ref faults) = faults1 {
            log_faults(1, faults);
        }
        if let Some(ref faults) = faults2 {
            log_faults(2, faults);
        }
        if let Some(ref faults) = faults3 {
            log_faults(3, faults);
        }
        if let Some(ref faults) = faults4 {
            log_faults(4, faults);
        }

        // Always print temperature readings (ADC counts)
        info!("Temps [ADC]: {} {} {} {}", tc1, tc2, tc3, tc4);

        // Store readings in batch
        packet.tc1_temps[batch_index] = tc1;
        packet.tc2_temps[batch_index] = tc2;
        packet.tc3_temps[batch_index] = tc3;
        packet.tc4_temps[batch_index] = tc4;

        batch_index += 1;

        // When batch is full, send UDP packet
        if batch_index >= BATCH_SIZE {
            packet.packet_tag = packet_counter;
            packet.packet_time = embassy_time::Instant::now().as_millis() as u32;

            // Send UDP packet
            match udp_socket.send_to(packet.as_bytes(), remote_endpoint).await {
                Ok(_) => {
                    info!(
                        "Sent packet #{} with {} readings",
                        packet_counter, BATCH_SIZE
                    );
                }
                Err(e) => {
                    info!("UDP send error: {:?}", e);
                }
            }

            packet_counter += 1;
            batch_index = 0;
        }

        Timer::after_millis(100).await;
    }
}
