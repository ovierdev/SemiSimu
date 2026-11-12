// Constante de Boltzman [J/K] (valor exacto SI 2019)
pub const K_B: f64 = 1.380649e-23;

// Carga elemental [C] (calor exacto SI 2019)
pub const Q: f64 = 1.6021765634e-19;

// Voltaje termico V_T = K_b*T/q [V]
pub fn thermal_voltage(t_kelvin: f64) -> f64 {
    K_B * t_kelvin / Q
}
