#![allow(dead_code)]
// MAX31856 Register Addresses (Read)
pub const CR0_READ: u8 = 0x00; // Configuration Register 0 (read)
pub const CR1_READ: u8 = 0x01; // Configuration Register 1 (read)
pub const MASK_READ: u8 = 0x02; // Fault Mask Register (read)
pub const CJHF_READ: u8 = 0x03; // Cold-Junction High Fault Threshold (read)
pub const CJLF_READ: u8 = 0x04; // Cold-Junction Low Fault Threshold (read)
pub const LTHFTH_READ: u8 = 0x05; // Linearized Temperature High Fault Threshold MSB (read)
pub const LTHFTL_READ: u8 = 0x06; // Linearized Temperature High Fault Threshold LSB (read)
pub const LTLFTH_READ: u8 = 0x07; // Linearized Temperature Low Fault Threshold MSB (read)
pub const LTLFTL_READ: u8 = 0x08; // Linearized Temperature Low Fault Threshold LSB (read)
pub const CJTO_READ: u8 = 0x09; // Cold-Junction Temperature Offset (read)
pub const CJTH_READ: u8 = 0x0A; // Cold-Junction Temperature MSB (read)
pub const CJTL_READ: u8 = 0x0B; // Cold-Junction Temperature LSB (read)
pub const LTCBH_READ: u8 = 0x0C; // Linearized TC Temperature, Byte 2 (MSB)
pub const LTCBM_READ: u8 = 0x0D; // Linearized TC Temperature, Byte 1
pub const LTCBL_READ: u8 = 0x0E; // Linearized TC Temperature, Byte 0 (LSB)
pub const SR_READ: u8 = 0x0F; // Fault Status Register (read)

// MAX31856 Register Addresses (Write)
pub const CR0_WRITE: u8 = 0x80; // Configuration Register 0 (write)
pub const CR1_WRITE: u8 = 0x81; // Configuration Register 1 (write)
pub const MASK_WRITE: u8 = 0x82; // Fault Mask Register (write)
pub const CJHF_WRITE: u8 = 0x83; // Cold-Junction High Fault Threshold (write)
pub const CJLF_WRITE: u8 = 0x84; // Cold-Junction Low Fault Threshold (write)
pub const LTHFTH_WRITE: u8 = 0x85; // Linearized Temperature High Fault Threshold MSB (write)
pub const LTHFTL_WRITE: u8 = 0x86; // Linearized Temperature High Fault Threshold LSB (write)
pub const LTLFTH_WRITE: u8 = 0x87; // Linearized Temperature Low Fault Threshold MSB (write)
pub const LTLFTL_WRITE: u8 = 0x88; // Linearized Temperature Low Fault Threshold LSB (write)
pub const CJTO_WRITE: u8 = 0x89; // Cold-Junction Temperature Offset (write)

// CR0 Bit Definitions
pub const CR0_FILTER_60HZ: u8 = 0; // 60Hz noise rejection (default)
pub const CR0_FILTER_50HZ: u8 = 1 << 0; // 50Hz noise rejection
pub const CR0_FAULTCLR: u8 = 1 << 1; // Fault status clear
pub const CR0_FAULT_INTERRUPT: u8 = 1 << 2; // Interrupt mode (vs comparator)
pub const CR0_CJ_DISABLED: u8 = 1 << 3; // Cold junction disabled
pub const CR0_CJ_ENABLED: u8 = 0; // Cold junction enabled (default)

// Open Circuit Fault Detection (bits 5:4)
pub const CR0_OC_DISABLED: u8 = 0; // Open circuit detection disabled (default)
pub const CR0_OC_ENABLED_RS_LT_5K: u8 = 1 << 4;
pub const CR0_OC_ENABLED_TC_LESS_2MS: u8 = 2 << 4; // 40k > RS > 5k, TC < 2ms
pub const CR0_OC_ENABLED_TC_MORE_2MS: u8 = 3 << 4; // 40k > RS > 5k, TC > 2ms

pub const CR0_ONESHOT: u8 = 1 << 6; // One-shot conversion
pub const CR0_CONV_NORMALLY_OFF: u8 = 0; // Normally off (default)
pub const CR0_CONV_CONTINUOUS: u8 = 1 << 7; // Automatic conversion mode

// CR1 Bit Definitions - Thermocouple Types (bits 3:0)
pub const CR1_TC_TYPE_B: u8 = 0x0;
pub const CR1_TC_TYPE_E: u8 = 0x1;
pub const CR1_TC_TYPE_J: u8 = 0x2;
pub const CR1_TC_TYPE_K: u8 = 0x3;
pub const CR1_TC_TYPE_N: u8 = 0x4;
pub const CR1_TC_TYPE_R: u8 = 0x5;
pub const CR1_TC_TYPE_S: u8 = 0x6;
pub const CR1_TC_TYPE_T: u8 = 0x7;

// CR1 Averaging Mode (bits 6:4)
pub const CR1_AVG_1_SAMPLE: u8 = 0 << 4;
pub const CR1_AVG_2_SAMPLES: u8 = 1 << 4;
pub const CR1_AVG_4_SAMPLES: u8 = 2 << 4;
pub const CR1_AVG_8_SAMPLES: u8 = 3 << 4;
pub const CR1_AVG_16_SAMPLES: u8 = 4 << 4;

// Fault Mask Register (0x02) Bit Definitions
// A value of 1 masks (disables) the fault, 0 unmasks (enables) it
pub const MASK_CJ_HIGH: u8 = 1 << 5; // Mask Cold-Junction High Fault
pub const MASK_CJ_LOW: u8 = 1 << 4; // Mask Cold-Junction Low Fault
pub const MASK_TC_HIGH: u8 = 1 << 3; // Mask Thermocouple High Fault
pub const MASK_TC_LOW: u8 = 1 << 2; // Mask Thermocouple Low Fault
pub const MASK_OVUV: u8 = 1 << 1; // Mask Overvoltage/Undervoltage Fault
pub const MASK_OPEN: u8 = 1 << 0; // Mask Open-Circuit Fault
pub const MASK_ALL_FAULTS: u8 = 0xFF; // Mask all faults (disable all fault detection)
pub const UNMASK_ALL_FAULTS: u8 = 0x00; // Unmask all faults (enable all fault detection)

// Fault Status Register Bit Definitions
pub const SR_CJ_RANGE: u8 = 1 << 7; // Cold-Junction Out-of-Range
pub const SR_TC_RANGE: u8 = 1 << 6; // Thermocouple Out-of-Range  
pub const SR_CJ_HIGH: u8 = 1 << 5; // Cold-Junction High Fault
pub const SR_CJ_LOW: u8 = 1 << 4; // Cold-Junction Low Fault
pub const SR_TC_HIGH: u8 = 1 << 3; // Thermocouple Temperature High Fault
pub const SR_TC_LOW: u8 = 1 << 2; // Thermocouple Temperature Low Fault
pub const SR_OVUV: u8 = 1 << 1; // Overvoltage or Undervoltage Input Fault
pub const SR_OPEN: u8 = 1 << 0; // Thermocouple Open-Circuit Fault
