//! Sistema de easing (curvas de aceleración) para animaciones.
//!
//! Cada función recibe `t` en `[0, 1]` y devuelve el valor easeado.

/// Identificador de función de easing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl Easing {
    /// Parsea un string a Easing (case-insensitive, acepta snake_case y camelCase).
    pub fn from_str(s: &str) -> Self {
        let normal = s.trim().to_lowercase().replace("-", "_").replace(" ", "_");
        match normal.as_str() {
            "linear" => Easing::Linear,
            "ease_in_quad" | "easeinquad" => Easing::EaseInQuad,
            "ease_out_quad" | "easeoutquad" => Easing::EaseOutQuad,
            "ease_in_out_quad" | "easeinoutquad" => Easing::EaseInOutQuad,
            "ease_in_cubic" | "easeincubic" => Easing::EaseInCubic,
            "ease_out_cubic" | "easeoutcubic" => Easing::EaseOutCubic,
            "ease_in_out_cubic" | "easeinoutcubic" => Easing::EaseInOutCubic,
            "ease_in_quart" | "easeinquart" => Easing::EaseInQuart,
            "ease_out_quart" | "easeoutquart" => Easing::EaseOutQuart,
            "ease_in_out_quart" | "easeinoutquart" => Easing::EaseInOutQuart,
            "ease_in_quint" | "easeinquint" => Easing::EaseInQuint,
            "ease_out_quint" | "easeoutquint" => Easing::EaseOutQuint,
            "ease_in_out_quint" | "easeinoutquint" => Easing::EaseInOutQuint,
            "ease_in_sine" | "easeinsine" => Easing::EaseInSine,
            "ease_out_sine" | "easeoutsine" => Easing::EaseOutSine,
            "ease_in_out_sine" | "easeinoutsine" => Easing::EaseInOutSine,
            "ease_in_expo" | "easeinexpo" => Easing::EaseInExpo,
            "ease_out_expo" | "easeoutexpo" => Easing::EaseOutExpo,
            "ease_in_out_expo" | "easeinoutexpo" => Easing::EaseInOutExpo,
            "ease_in_circ" | "easeincirc" => Easing::EaseInCirc,
            "ease_out_circ" | "easeoutcirc" => Easing::EaseOutCirc,
            "ease_in_out_circ" | "easeinoutcirc" => Easing::EaseInOutCirc,
            "ease_in_elastic" | "easeinelastic" => Easing::EaseInElastic,
            "ease_out_elastic" | "easeoutelastic" => Easing::EaseOutElastic,
            "ease_in_out_elastic" | "easeinoutelastic" => Easing::EaseInOutElastic,
            "ease_in_back" | "easeinback" => Easing::EaseInBack,
            "ease_out_back" | "easeoutback" => Easing::EaseOutBack,
            "ease_in_out_back" | "easeinoutback" => Easing::EaseInOutBack,
            "ease_in_bounce" | "easeinbounce" => Easing::EaseInBounce,
            "ease_out_bounce" | "easeoutbounce" => Easing::EaseOutBounce,
            "ease_in_out_bounce" | "easeinoutbounce" => Easing::EaseInOutBounce,
            _ => Easing::Linear, // default
        }
    }
}

/// Aplica la función de easing a `t` (debe estar en `[0, 1]`).
/// Devuelve el valor easeado, normalmente también en `[0, 1]` (excepto back/elastic que pueden overshoot).
pub fn apply(easing: &Easing, t: f64) -> f64 {
    match easing {
        Easing::Linear => t,
        Easing::EaseInQuad => t * t,
        Easing::EaseOutQuad => t * (2.0 - t),
        Easing::EaseInOutQuad => {
            if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
        }
        Easing::EaseInCubic => t * t * t,
        Easing::EaseOutCubic => {
            let t = t - 1.0;
            t * t * t + 1.0
        }
        Easing::EaseInOutCubic => {
            if t < 0.5 { 4.0 * t * t * t } else { (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0 }
        }
        Easing::EaseInQuart => t * t * t * t,
        Easing::EaseOutQuart => {
            let t = t - 1.0;
            -(t * t * t * t - 1.0)
        }
        Easing::EaseInOutQuart => {
            if t < 0.5 { 8.0 * t * t * t * t } else {
                let t = t - 1.0;
                1.0 - 8.0 * t * t * t * t
            }
        }
        Easing::EaseInQuint => t * t * t * t * t,
        Easing::EaseOutQuint => {
            let t = t - 1.0;
            t * t * t * t * t + 1.0
        }
        Easing::EaseInOutQuint => {
            if t < 0.5 { 16.0 * t * t * t * t * t } else {
                let t = t - 1.0;
                1.0 + 16.0 * t * t * t * t * t
            }
        }
        Easing::EaseInSine => 1.0 - (t * std::f64::consts::FRAC_PI_2).cos(),
        Easing::EaseOutSine => (t * std::f64::consts::FRAC_PI_2).sin(),
        Easing::EaseInOutSine => 0.5 * (1.0 - (t * std::f64::consts::PI).cos()),
        Easing::EaseInExpo => {
            if t == 0.0 { 0.0 } else { (2.0_f64).powf(10.0 * (t - 1.0)) }
        }
        Easing::EaseOutExpo => {
            if t == 1.0 { 1.0 } else { 1.0 - (2.0_f64).powf(-10.0 * t) }
        }
        Easing::EaseInOutExpo => {
            if t == 0.0 { 0.0 }
            else if t == 1.0 { 1.0 }
            else if t < 0.5 { 0.5 * (2.0_f64).powf(10.0 * (2.0 * t - 1.0)) }
            else { 0.5 * (2.0 - (2.0_f64).powf(-10.0 * (2.0 * t - 1.0))) }
        }
        Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
        Easing::EaseOutCirc => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
        Easing::EaseInOutCirc => {
            if t < 0.5 {
                0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
            } else {
                let t = 2.0 * t - 2.0;
                0.5 * ((1.0 - t * t).sqrt() + 1.0)
            }
        }
        Easing::EaseInElastic => {
            if t == 0.0 || t == 1.0 { t }
            else {
                let t = t - 1.0;
                -( (2.0_f64).powf(10.0 * t) * ((t * 20.0 - 11.0) * std::f64::consts::FRAC_PI_2 * 1.5 / 2.5).sin() )
            }
        }
        Easing::EaseOutElastic => {
            if t == 0.0 || t == 1.0 { t }
            else {
                (2.0_f64).powf(-10.0 * t) * ((t * 20.0 - 11.0) * std::f64::consts::FRAC_PI_2 * 1.5 / 2.5).sin() + 1.0
            }
        }
        Easing::EaseInOutElastic => {
            if t == 0.0 || t == 1.0 { t }
            else if t < 0.5 {
                let t = 2.0 * t - 1.0;
                -0.5 * (2.0_f64).powf(10.0 * t) * ((t * 20.0 - 11.0) * std::f64::consts::FRAC_PI_2 * 1.5 / 2.5).sin()
            } else {
                let t = 2.0 * t - 1.0;
                0.5 * (2.0_f64).powf(-10.0 * t) * ((t * 20.0 - 11.0) * std::f64::consts::FRAC_PI_2 * 1.5 / 2.5).sin() + 1.0
            }
        }
        Easing::EaseInBack => {
            let s = 1.70158;
            t * t * ((s + 1.0) * t - s)
        }
        Easing::EaseOutBack => {
            let s = 1.70158;
            let t = t - 1.0;
            t * t * ((s + 1.0) * t + s) + 1.0
        }
        Easing::EaseInOutBack => {
            let s = 1.70158 * 1.525;
            if t < 0.5 {
                0.5 * (2.0 * t * 2.0 * t * ((s + 1.0) * 2.0 * t - s))
            } else {
                let t = 2.0 * t - 2.0;
                0.5 * (t * t * ((s + 1.0) * t + s) + 2.0)
            }
        }
        Easing::EaseInBounce => 1.0 - ease_out_bounce(1.0 - t),
        Easing::EaseOutBounce => ease_out_bounce(t),
        Easing::EaseInOutBounce => {
            if t < 0.5 {
                0.5 * (1.0 - ease_out_bounce(1.0 - 2.0 * t))
            } else {
                0.5 * ease_out_bounce(2.0 * t - 1.0) + 0.5
            }
        }
    }
}

fn ease_out_bounce(t: f64) -> f64 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        7.5625 * t * t + 0.75
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        7.5625 * t * t + 0.9375
    } else {
        let t = t - 2.625 / 2.75;
        7.5625 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert!((apply(&Easing::Linear, 0.0) - 0.0).abs() < 1e-10);
        assert!((apply(&Easing::Linear, 0.5) - 0.5).abs() < 1e-10);
        assert!((apply(&Easing::Linear, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_boundaries() {
        for e in &[Easing::EaseInCubic, Easing::EaseOutSine, Easing::EaseInOutQuad,
                   Easing::EaseInExpo, Easing::EaseOutCirc, Easing::EaseInBack] {
            let v0 = apply(e, 0.0);
            let v1 = apply(e, 1.0);
            assert!((v0 - 0.0).abs() < 1e-5 || v0 < 0.0, "{:?} at 0 = {}", e, v0);
            assert!((v1 - 1.0).abs() < 1e-5 || v1 > 1.0, "{:?} at 1 = {}", e, v1);
        }
    }

    #[test]
    fn test_parse() {
        assert_eq!(Easing::from_str("linear"), Easing::Linear);
        assert_eq!(Easing::from_str("ease_out_cubic"), Easing::EaseOutCubic);
        assert_eq!(Easing::from_str("easeOutElastic"), Easing::EaseOutElastic);
        assert_eq!(Easing::from_str("EASE_IN_BOUNCE"), Easing::EaseInBounce);
        assert_eq!(Easing::from_str("ease-in-out-quart"), Easing::EaseInOutQuart);
    }
}
