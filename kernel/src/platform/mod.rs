use driver::Driver;

pub mod mpu;
pub mod systick;

/// Interface for individual boards.
pub trait Platform {
    /// Platform-specific mapping of syscall numbers to objects that implement
    /// the Driver methods for that syscall
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R where F: FnOnce(Option<&Driver>) -> R;
}

/// Interface for individual MCUs.
pub trait Chip {
    type MPU: mpu::MPU;
    type SysTick: systick::SysTick;

    fn service_pending_interrupts(&mut self);
    fn has_pending_interrupts(&self) -> bool;
    fn mpu(&self) -> &Self::MPU;
    fn systick(&self) -> &Self::SysTick;
    fn prepare_for_sleep(&self) {}
}

/// Generic operations that clock-like things are expected to support.
pub trait ClockInterface {
    type PlatformClockType;

    fn is_enabled(&self) -> bool;
    fn enable(&self);
    fn disable(&self);
}


//use core::marker::PhantomData;

pub trait MMIOInterface<C> where
    C: ?Sized + ClockInterface,
{
    type MMIORegisterType : ?Sized;
    type MMIOClockType : ?Sized + ClockInterface;

    fn get_hardware_address(&self) -> *mut Self::MMIORegisterType;
    fn get_clock(&self) -> &C;
    fn can_disable_clock(&self, &Self::MMIORegisterType) -> bool;
}

pub struct MMIOManager<'a, H, R, C> where
    H: 'a + ?Sized + MMIOInterface<C, MMIORegisterType=R>,
    R: 'a + ?Sized,
    C: 'a + ?Sized + ClockInterface,
{
    pub registers: &'a R,
    clock: &'a C,
    periphal_hardware: &'a H,
}

impl<'a, H, R, C> MMIOManager<'a, H, R, C> where
    H: 'a + ?Sized + MMIOInterface<C, MMIORegisterType=R>,
    R: 'a + ?Sized,
    C: 'a + ?Sized + ClockInterface,
{
    pub fn new(hw: &'a H) -> MMIOManager<'a, H, R, C> {
        let clock = hw.get_clock();
        if clock.is_enabled() == false {
            clock.enable();
        }
        MMIOManager {
            registers: unsafe { &* hw.get_hardware_address() },
            clock: hw.get_clock(),
            periphal_hardware: hw,
        }
    }
}

impl<'a, H, R, C> Drop for MMIOManager<'a, H, R, C> where
    H: 'a + ?Sized + MMIOInterface<C, MMIORegisterType=R>,
    R: 'a + ?Sized,
    C: 'a + ?Sized + ClockInterface,
{
    fn drop(&mut self) {
        if self.periphal_hardware.can_disable_clock(self.registers) {
            self.clock.disable();
        }
    }
}
