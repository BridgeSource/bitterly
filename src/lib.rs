#![no_std]

/// # Bitterly
///
/// Bitterly is a simple, macro based Rust library used to generate a peripheral
/// and is intented to be used with embedded systems that interface to devices
/// on a bus such as I2C or SPI where results can be cache and stored in RAM.
///
/// It can be useful to read multiple registers, interact with the data, and
/// then write or transmit the results back to the peripheral or another device
/// (host pc?). This library provides an easy way to interact with the registers
/// of a peripheral.
///
///
///

/// The register_backer! macro is used to generate a RegisterBacker struct that is used
/// by subsequent macros, such as peripheral!. The generated struct has accessors
/// to get / set / toggle / clear bits, as well as get a range of bits.
#[macro_export]
macro_rules! register_backer {
    ($reg_name:ident, $reg_type:ty) => {
        type RegisterType = $reg_type;

        type RegisterBacker = $reg_name;

        #[derive(Copy, Clone)]
        pub struct $reg_name {
            contents: $reg_type,
        }

        #[derive(Copy, Clone)]
        pub struct BitRange {
            start_bit: $reg_type,
            stop_bit: $reg_type,
        }

        impl BitRange {
            pub fn new(start_bit: $reg_type, stop_bit: $reg_type) -> Self {
                assert!(
                    start_bit <= stop_bit,
                    "Start bit must be less than or equal to stop bit"
                );
                BitRange {
                    start_bit,
                    stop_bit,
                }
            }
        }

        impl $reg_name {
            pub fn new(contents: $reg_type) -> Self {
                Self { contents: contents }
            }

            pub fn contents(&self) -> $reg_type {
                self.contents
            }

            pub fn set_bit(&mut self, bit: $reg_type) -> &mut Self {
                self.contents |= 1 << (bit as $reg_type);
                self
            }

            pub fn set_all(&mut self) -> &mut Self {
                self.contents = <$reg_type>::MAX;
                self
            }

            pub fn clear_bit(&mut self, bit: $reg_type) -> &mut Self {
                self.contents &= !(1 << (bit as $reg_type));
                self
            }

            pub fn clear_all(&mut self) -> &mut Self {
                self.contents = 0;
                self
            }

            pub fn toggle_bit(&mut self, bit: $reg_type) -> &mut Self {
                self.contents ^= 1 << (bit as $reg_type);
                self
            }

            pub fn is_set(&self, bit: $reg_type) -> bool {
                self.contents & (1 << (bit as $reg_type)) != 0
            }

            pub fn is_clear(&self, bit: $reg_type) -> bool {
                self.contents & (1 << (bit as $reg_type)) == 0
            }

            pub fn update(&mut self, new_val: $reg_type) -> &mut Self {
                self.contents = new_val;
                self
            }

            pub fn get_range(&self, range: BitRange) -> $reg_type {
                ((self.contents & self.mask(range)) >> range.start_bit) as $reg_type
            }

            pub fn mask(&self, br: BitRange) -> $reg_type {
                let exp = (2 as $reg_type).overflowing_pow((br.stop_bit + 1 - br.start_bit) as u32);
                if exp.1 == true {
                    <$reg_type>::MAX << br.start_bit
                } else {
                    (exp.0 - 1) << br.start_bit
                }
            }

            pub fn clear_range(&mut self, range: BitRange) -> &mut Self {
                self.contents = self.contents & !self.mask(range);
                self
            }

            pub fn set_range(&mut self, range: BitRange, val: $reg_type) -> &mut Self {
                self.clear_range(range); // Clear bits
                let masked_val = self.mask(range) & (val << range.start_bit); // Mask input
                self.contents = self.contents | masked_val;
                self
            }
        }
    };
}

/// This macro is used to generate a struct called a peripheral that allocates
/// a number of registers that can be pointed to using the subsequent macros. The
/// macro takes three arguments, the name of the peripheral, the number of registers
/// that the peripheral has, and a list of tuples that contain the name of the register.
///
/// Note, the number of registers should be >= the number of registers in the list of tuples.
/// or there will be a panic at runtime if you try to access a register that is outside of
/// the range of the number of registers.
#[macro_export]
macro_rules! peripheral {
    //($enum_name:ident, $enum_type:ty, [$(($name:ident, $value:literal)),+]) => {

    ($peripheral_name:ident, $count:literal, [$(($register:ident, $addr:literal)),+]) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum RegisterId {
            $(
                $register = $addr,
            )+
        }

        pub struct $peripheral_name {
            registers: [RegisterBacker; $count],
        }

        impl $peripheral_name {
            pub fn new() -> Self {
                $peripheral_name {
                    registers: [RegisterBacker { contents: 0 }; $count],
                }
            }
        }

        type PeripheralType = $peripheral_name;
    };

}

/// This macro is used to generate a struct that contains a pointer to a register. This is useful
/// when you have a peripheral that has multiple registers that you want to access. The macro
/// generates a struct with the name provided, and a function with the name provided that returns.
///
/// This function requires that a peripheral has been created using the
/// peripheral! macro so that that pointers to the backing memory can be used.
#[macro_export]
macro_rules! register {
    ($register:ident) => {
        pub struct $register {
            register: *mut Register,
        }

        impl $register {
            pub fn contents(&self) -> RegisterType {
                unsafe { (*self.register).contents }
            }

            pub fn address(&self) -> usize {
                RegisterId::$register as usize as usize
            }

            pub fn update(&mut self, val: RegisterType) -> &mut Self {
                unsafe {
                    (*self.register).contents = val;
                }
                self
            }

            pub fn clear_all(&mut self) -> &mut Self {
                unsafe {
                    (*self.register).clear_all();
                }
                self
            }
        }

        impl PeripheralType {
            pub fn $register(&self) -> $register {
                $register {
                    register: &self.registers[RegisterId::$register as usize] as *const _
                        as *mut Register,
                }
            }
        }
    };
}

/// This macro generates a bitfield within a register for a single bit. For example,
/// if a register contains a bit flag, this could be used to get / set the bit
/// flag in a named way.
#[macro_export]
macro_rules! bitfield {
    ($register:ident, $bitfield_name:ident, $bit:literal) => {
        paste! {
            pub trait $bitfield_name {
                fn [<get_ $bitfield_name>](&self) -> bool;
                fn [<set_ $bitfield_name>](&mut self, value: bool) -> &mut Self;
            }

            impl $bitfield_name for $register {
                fn [<get_ $bitfield_name>](&self) -> bool {
                    unsafe {
                        self.register.as_mut().unwrap().is_set($bit)
                    }
                }

                fn [<set_ $bitfield_name>](&mut self, value: bool) -> &mut Self {
                    unsafe {
                        if value {
                            self.register.as_mut().unwrap().set_bit($bit);
                        } else {
                            self.register.as_mut().unwrap().clear_bit($bit);
                        }
                    }

                    self
                }
            }
        }
    };
}

/// Generates an enumerated type for a bitrange and should be done BEFORE the
/// bitrange register is used to ensure the correct types are in place. This macro
/// also takes care of converting the enum to a number and vice versa.
#[macro_export]
macro_rules! bitrange_enum_values {
    ($enum_name:ident, $enum_type:ty, [$(($name:ident, $value:literal)),+]) => {
        paste! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            pub enum $enum_name {
                $(
                    $name = $value,
                )+
            }

            pub fn [<$enum_name FromNum>](num: $enum_type) -> Option<$enum_name> {
                match num {
                    $(
                        $value => Some($enum_name::$name),
                    )+
                    _ => None,
                }
            }

            pub fn [<$enum_name ToNum>](enum_value: $enum_name) -> $enum_type {
                enum_value as $enum_type
            }
        }
    };
}

/// Defines a bitrange and the correct getters and setters for the bitrange using
/// an enum defined using the bitrange_enum_values macro. This helps enforce
/// only valid enum can be used for the getters and setters. If a register
/// doesn't have an enum that is easily mappable, consider bitrange_raw instead.
#[macro_export]
macro_rules! bitrange {
    ($register:ident, $bitrange_name:ident, $msb:literal, $lsb:literal, $val_type:ty) => {
        paste! {
            pub trait $bitrange_name {
                fn [<get_ $bitrange_name>](&self) -> Option<$val_type>;
                fn [<set_ $bitrange_name>](&mut self, value: $val_type) -> &mut Self;
            }

            impl $bitrange_name for $register {
                fn [<get_ $bitrange_name>](&self) -> Option<$val_type> {
                    unsafe {
                        let val = self.register.as_mut().unwrap().get_range(BitRange {stop_bit: $msb, start_bit: $lsb });
                        [<$val_type FromNum>](val)
                    }
                }

                fn [<set_ $bitrange_name>](&mut self, value: $val_type) -> &mut Self {
                    unsafe {
                        self.register.as_mut().unwrap().set_range(BitRange { stop_bit: $msb, start_bit: $lsb }, [<$val_type ToNum>](value));
                    }

                    self
                }
            }
        }
    };
}

/// Defines a bitrange and the correct getters and setters for the bitrange using
/// a raw type, such as u8, u16, u32, etc. This can be used for registers like
/// ID and Revision that don't map to an enum but occupy a bitrange.
#[macro_export]
macro_rules! bitrange_raw {
    ($register:ident, $bitrange_name:ident, $msb:literal, $lsb:literal, $val_type:ty) => {
        paste! {
            trait $bitrange_name {
                fn [<get_ $bitrange_name>](&self) -> $val_type;
                fn [<set_ $bitrange_name>](&mut self, value: $val_type) -> &mut Self;
            }

            impl $bitrange_name for $register {
                fn [<get_ $bitrange_name>](&self) -> $val_type {
                    unsafe {
                        self.register.as_mut().unwrap().get_range(BitRange {stop_bit: $msb, start_bit: $lsb })
                    }
                }

                fn [<set_ $bitrange_name>](&mut self, value: $val_type) -> &mut Self {
                    unsafe {
                        self.register.as_mut().unwrap().set_range(BitRange { stop_bit: $msb, start_bit: $lsb }, value);
                    }

                    self
                }
            }
        }
    };
}
