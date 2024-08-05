#[inline]
pub fn cpu_relax() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // asm volatile("yield" ::: "memory")
        core::arch::asm!("yield", options(nomem))
    }
}
