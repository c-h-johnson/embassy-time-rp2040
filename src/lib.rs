#![no_std]

use core::cell::RefCell;

use critical_section::Mutex;
use embassy_time::driver::{AlarmHandle, Driver};
use rp2040_hal::{
    pac::interrupt,
    timer::{Alarm, Alarm0, Alarm1, Alarm2, Alarm3, Instant, ScheduleAlarmError, Timer},
};

embassy_time::time_driver_impl!(static DRIVER: Rp2040TimeDriver = Rp2040TimeDriver::new());

enum Rp2040Alarm {
    A0(Alarm0),
    A1(Alarm1),
    A2(Alarm2),
    A3(Alarm3),
}

impl Rp2040Alarm {
    fn clear_interrupt(&mut self) {
        match self {
            Rp2040Alarm::A0(alarm) => alarm.clear_interrupt(),
            Rp2040Alarm::A1(alarm) => alarm.clear_interrupt(),
            Rp2040Alarm::A2(alarm) => alarm.clear_interrupt(),
            Rp2040Alarm::A3(alarm) => alarm.clear_interrupt(),
        }
    }

    fn enable_interrupt(&mut self) {
        match self {
            Rp2040Alarm::A0(alarm) => alarm.enable_interrupt(),
            Rp2040Alarm::A1(alarm) => alarm.enable_interrupt(),
            Rp2040Alarm::A2(alarm) => alarm.enable_interrupt(),
            Rp2040Alarm::A3(alarm) => alarm.enable_interrupt(),
        }
    }

    fn schedule_at(&mut self, timestamp: Instant) -> Result<(), ScheduleAlarmError> {
        match self {
            Rp2040Alarm::A0(alarm) => alarm.schedule_at(timestamp),
            Rp2040Alarm::A1(alarm) => alarm.schedule_at(timestamp),
            Rp2040Alarm::A2(alarm) => alarm.schedule_at(timestamp),
            Rp2040Alarm::A3(alarm) => alarm.schedule_at(timestamp),
        }
    }
}

#[derive(Copy, Clone)]
struct Callback {
    callback: fn(*mut ()),
    ctx: *mut (),
}
unsafe impl Send for Callback {}

struct Rp2040TimeDriver {
    timer: Mutex<RefCell<Option<Timer>>>,
    alarm_allocated: Mutex<RefCell<[bool; 4]>>,
    alarms: Mutex<RefCell<Option<[Rp2040Alarm; 4]>>>,
    callbacks: Mutex<RefCell<[Option<Callback>; 4]>>,
}

impl Rp2040TimeDriver {
    const fn new() -> Self {
        Self {
            timer: Mutex::new(RefCell::new(None)),
            alarm_allocated: Mutex::new(RefCell::new([false; 4])),
            alarms: Mutex::new(RefCell::new(None)),
            callbacks: Mutex::new(RefCell::new([None; 4])),
        }
    }

    fn init(&self, mut timer: Timer) {
        let alarms = [
            Rp2040Alarm::A0(timer.alarm_0().unwrap()),
            Rp2040Alarm::A1(timer.alarm_1().unwrap()),
            Rp2040Alarm::A2(timer.alarm_2().unwrap()),
            Rp2040Alarm::A3(timer.alarm_3().unwrap()),
        ];
        critical_section::with(|cs| {
            let mut borrowed_timer = self.timer.borrow_ref_mut(cs);
            let _ = borrowed_timer.insert(timer);
            let mut borrowed_alarms = self.alarms.borrow_ref_mut(cs);
            let _ = borrowed_alarms.insert(alarms);
        })
    }

    fn interrupt(&self, alarm_index: usize) {
        critical_section::with(|cs| {
            let callbacks = self.callbacks.borrow_ref_mut(cs);
            let cb = callbacks[alarm_index].unwrap();
            let f = cb.callback;
            f(cb.ctx);

            let mut alarms_borrowed = self.alarms.borrow_ref_mut(cs);
            let alarms = alarms_borrowed.as_mut().unwrap();
            let alarm = &mut alarms[alarm_index];
            alarm.clear_interrupt();
        })
    }
}

impl Driver for Rp2040TimeDriver {
    fn now(&self) -> u64 {
        critical_section::with(|cs| {
            let mut timer = self.timer.borrow_ref_mut(cs);
            timer.as_mut().unwrap().get_counter().ticks()
        })
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|cs| {
            let mut alarm_handle = None;
            let mut alarm_allocated = self.alarm_allocated.borrow_ref_mut(cs);
            for (index, alarm) in alarm_allocated.iter().enumerate() {
                if !alarm {
                    alarm_handle = Some(AlarmHandle::new(index as u8));
                    break;
                }
            }

            if let Some(found_alarm_handle) = alarm_handle {
                alarm_allocated[found_alarm_handle.id() as usize] = true;
            }

            alarm_handle
        })
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        critical_section::with(|cs| {
            let mut callbacks = self.callbacks.borrow_ref_mut(cs);
            callbacks[alarm.id() as usize] = Some(Callback { callback, ctx });
        })
    }

    fn set_alarm(&self, alarm_handle: AlarmHandle, timestamp: u64) -> bool {
        let instant = Instant::from_ticks(timestamp);
        critical_section::with(|cs| {
            let mut alarms_borrowed = self.alarms.borrow_ref_mut(cs);
            let alarms = alarms_borrowed.as_mut().unwrap();
            let alarm = &mut alarms[alarm_handle.id() as usize];
            alarm.enable_interrupt();
            alarm.schedule_at(instant).is_ok()
        })
    }
}

/// only call this once
pub fn init(timer: Timer) {
    DRIVER.init(timer);
}

#[interrupt]
fn TIMER_IRQ_0() {
    DRIVER.interrupt(0);
}

#[interrupt]
fn TIMER_IRQ_1() {
    DRIVER.interrupt(1);
}

#[interrupt]
fn TIMER_IRQ_2() {
    DRIVER.interrupt(2);
}

#[interrupt]
fn TIMER_IRQ_3() {
    DRIVER.interrupt(3);
}
