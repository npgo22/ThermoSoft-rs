#![no_std]
#![allow(non_snake_case)] // Allow non-snake-case crate name (ThermoSoft-rs)

pub mod max31856;

use embedded_hal::spi::SpiDevice;
use max31856::FaultStatus;
use max31856::registers::*;

// Packet batching configuration
pub const SENSOR_COUNT: usize = 4;
pub const BATCH_SIZE: usize = 10;

/// Packed structure for batched sensor data packet
/// Matches the C structure layout for network transmission
/// Also by default the rust compiler will move your fields around
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SensorDataPacket {
    pub packet_tag: u32,              // Packet identifier
    pub tc1_temps: [i32; BATCH_SIZE], // Thermocouple 1 temperature batch
    pub tc2_temps: [i32; BATCH_SIZE], // Thermocouple 2 temperature batch
    pub tc3_temps: [i32; BATCH_SIZE], // Thermocouple 3 temperature batch
    pub tc4_temps: [i32; BATCH_SIZE], // Thermocouple 4 temperature batch
    pub packet_time: u32,             // Timestamp when packet was sent (milliseconds)
}

impl Default for SensorDataPacket {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorDataPacket {
    /// Create a new empty packet
    pub const fn new() -> Self {
        Self {
            packet_tag: 0,
            tc1_temps: [0; BATCH_SIZE],
            tc2_temps: [0; BATCH_SIZE],
            tc3_temps: [0; BATCH_SIZE],
            tc4_temps: [0; BATCH_SIZE],
            packet_time: 0,
        }
    }

    /// Convert packet to byte slice for transmission
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// Log faults for a sensor
pub fn log_faults(sensor_num: u8, faults: &FaultStatus) {
    if faults.open {
        defmt::warn!("Sensor {} - Open circuit fault", sensor_num);
    }
    if faults.ovuv {
        defmt::warn!("Sensor {} - Over/Under voltage fault", sensor_num);
    }
    // if faults.cj_range {
    //     defmt::warn!("Sensor {} - Cold junction out of range", sensor_num);
    // }
    // if faults.tc_range {
    //     defmt::warn!("Sensor {} - Thermocouple out of range", sensor_num);
    // }
    if faults.cj_high {
        defmt::warn!("Sensor {} - Cold junction high fault", sensor_num);
    }
    if faults.cj_low {
        defmt::warn!("Sensor {} - Cold junction low fault", sensor_num);
    }
    if faults.tc_high {
        defmt::warn!("Sensor {} - Thermocouple high fault", sensor_num);
    }
    if faults.tc_low {
        defmt::warn!("Sensor {} - Thermocouple low fault", sensor_num);
    }
}

/// Configure MAX31856 with application-specific settings
pub fn configure_max31856<SPI>(spi: &mut SPI) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    let cr0_config = CR0_FILTER_60HZ
        | CR0_FAULT_INTERRUPT
        | CR0_CJ_ENABLED
        | CR0_OC_ENABLED_RS_LT_5K
        | CR0_CONV_CONTINUOUS;

    spi.write(&[CR0_WRITE, cr0_config])?;

    let cr1_config = CR1_TC_TYPE_K | CR1_AVG_4_SAMPLES;

    spi.write(&[CR1_WRITE, cr1_config])?;

    // Unmask all faults - let all fault conditions be reported
    spi.write(&[MASK_WRITE, 0x00])?;

    // Set Cold-Junction fault thresholds (-55°C to +85°C - typical IC operating range)
    max31856::set_cj_low_fault_threshold(spi, -55)?;
    max31856::set_cj_high_fault_threshold(spi, 85)?;

    // Set Thermocouple fault thresholds to maximum range
    max31856::set_tc_low_fault_threshold(spi, -270.0)?;
    max31856::set_tc_high_fault_threshold(spi, 1372.0)?;

    Ok(())
}

/// Configure and verify a MAX31856 sensor with detailed logging
pub fn configure_and_verify_max31856<SPI>(spi: &mut SPI, sensor_num: u8) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    // Configure the sensor
    configure_max31856(spi)?;

    // Read back and verify configuration
    let regs = max31856::read_all_config_registers(spi)?;

    defmt::info!(
        "Sensor {} - CR0={:02X} CR1={:02X} MASK={:02X} SR={:02X}",
        sensor_num,
        regs[0],
        regs[1],
        regs[2],
        regs[15]
    );
    defmt::info!(
        "Sensor {} - CJ thresholds: Low={:02X} High={:02X}",
        sensor_num,
        regs[4],
        regs[3]
    );
    defmt::info!(
        "Sensor {} - TC thresholds: Low={:02X}{:02X} High={:02X}{:02X}",
        sensor_num,
        regs[7],
        regs[8],
        regs[5],
        regs[6]
    );

    Ok(())
}
