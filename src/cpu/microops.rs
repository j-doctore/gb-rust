pub const T_CYCLES_PER_M_CYCLE: u32 = 4;

#[inline]
pub fn m_cycles(n: u32) -> u32 {
	n * T_CYCLES_PER_M_CYCLE
}

#[inline]
pub fn bus_access() -> u32 {
	m_cycles(1)
}

#[inline]
pub fn halt_idle() -> u32 {
	m_cycles(1)
}

