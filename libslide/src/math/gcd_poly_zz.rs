use crate::math::gcd;
use crate::math::poly::Poly;
use crate::poly;

use core::cmp::{max, min};

/// Calculates the GCD of two polynomials, `f` and `g`, in ZZ (integer, integer) space using the
/// GCDHEU heuristic algorithm (sources: [1], [2]).
///
/// If successful, the algorithm returns the tuple (gcd, f_quotient, g_quotient). If the algorithm
/// fails (either because the heuristic terminates or another reason) an error is returned.
///
/// [1]: https://www.sciencedirect.com/science/article/pii/S0747717189800045
/// [2]: https://dl.acm.org/doi/pdf/10.1145/220346.220376
pub fn gcd_poly_zz_heu(f: Poly, g: Poly) -> Result<(Poly, Poly, Poly), &'static str> {
    if let Some(res) = trivial_gcd(&f, &g) {
        return Ok(res);
    }

    let df = f.deg();
    let dg = f.deg();

    let (gcd_val, f, g) = poly_extract_common(f, g);

    // Handle single-degree cases.
    if df == 1 || dg == 1 {
        return Ok((Poly::empty(), f, g));
    }
    let f_norm = f.max_norm();
    let g_norm = g.max_norm();

    let norm_char = 2 * min(f_norm, g_norm) + 29;
    let mut xi: isize = max(
        min(norm_char, 10_000 * (norm_char as f64).sqrt() as usize / 101),
        2 * min(
            f_norm / f.lc().abs() as usize,
            g_norm / g.lc().abs() as usize + 2,
        ),
    ) as isize;

    const GCD_HEU_MAX_ITER: u8 = 6;
    let mut f_xi;
    let mut g_xi;
    let mut heu;
    let mut heu_poly;
    let mut cff;
    let mut cfg;
    let mut cff_poly;
    let mut cfg_poly;
    let mut _cff2;
    let mut _cfg2;
    let mut r;
    for _ in 0..GCD_HEU_MAX_ITER {
        f_xi = f.eval(xi as isize);
        g_xi = g.eval(xi as isize);

        if f_xi != 0 && g_xi != 0 {
            heu = gcd(f_xi.abs() as usize, g_xi.abs() as usize) as isize;
            cff = f_xi / heu;
            cfg = g_xi / heu;

            heu_poly = gcd_interpolate(heu, xi);
            heu_poly = heu_poly.primitive();

            match f.clone().div(heu_poly.clone()) {
                Ok(q) => {
                    _cff2 = q.0;
                    r = q.1
                }
                Err(e) => return Err(e),
            }
            if r.is_empty() {
                match g.clone().div(heu_poly.clone()) {
                    Ok(q) => {
                        _cfg2 = q.0;
                        r = q.1;
                    }
                    Err(e) => return Err(e),
                }
                if r.is_empty() {
                    heu_poly = heu_poly.mul_scalar(gcd_val);
                    return Ok((heu_poly, _cff2, _cfg2));
                }
            }
            cff_poly = gcd_interpolate(cff, xi);
            match f.clone().div(cff_poly.clone()) {
                Ok(q) => {
                    heu_poly = q.0;
                    r = q.1;
                }
                Err(e) => return Err(e),
            }
            if r.is_empty() {
                match g.clone().div(heu_poly.clone()) {
                    Ok(q) => {
                        _cfg2 = q.0;
                        r = q.1;
                    }
                    Err(e) => return Err(e),
                }
                if r.is_empty() {
                    heu_poly = heu_poly.mul_scalar(gcd_val);
                    return Ok((heu_poly, cff_poly, _cfg2));
                }
            }
            cfg_poly = gcd_interpolate(cfg, xi);
            match g.clone().div(cfg_poly.clone()) {
                Ok(q) => {
                    heu_poly = q.0;
                    r = q.1;
                }
                Err(e) => return Err(e),
            }
            if r.is_empty() {
                match f.clone().div(heu_poly.clone()) {
                    Ok(q) => {
                        _cff2 = q.0;
                        r = q.1;
                    }
                    Err(e) => return Err(e),
                }
                if r.is_empty() {
                    heu_poly = heu_poly.mul_scalar(gcd_val);
                    return Ok((heu_poly, _cff2, cfg_poly));
                }
            }
        }
        xi = 73794 * xi * ((xi as f64).sqrt().sqrt() as isize)
    }

    Err("gcd_poly_zz_heu failed")
}

#[cfg(feature = "benchmark-internals")]
pub fn _gcd_poly_zz_heu<T, U>(f: T, g: U) -> Result<(Poly, Poly, Poly), &'static str>
where
    T: Into<Poly>,
    U: Into<Poly>,
{
    gcd_poly_zz_heu(f.into(), g.into())
}

/// Returns the GCD of a polynomial's term coefficients.
fn poly_coeffs_gcd(p: &Poly) -> usize {
    if p.is_empty() {
        return 0;
    }
    let mut cont: usize = p.vec[0] as usize;
    for i in &p.vec {
        cont = gcd(cont, i.abs() as usize);
        if cont == 1 {
            break;
        }
    }
    cont as usize
}

/// Extracts a constant coefficient from two polynomials.
/// Returns a tuple of (constant, f_quotient, g_quotient).
fn poly_extract_common(mut f: Poly, mut g: Poly) -> (isize, Poly, Poly) {
    // extracts 'Common gcd content' from polys
    let fc = poly_coeffs_gcd(&f);
    let gc = poly_coeffs_gcd(&g);
    let gcd = gcd(fc, gc) as isize;

    if gcd == 0 {
        (1, f, g)
    } else {
        f = f.div_scalar(gcd).unwrap();
        g = g.div_scalar(gcd).unwrap();
        (gcd, f, g)
    }
}

/// Handles trivial polynomial GCD cases, namely if one polynomial is empty.
fn trivial_gcd(f: &Poly, g: &Poly) -> Option<(Poly, Poly, Poly)> {
    match (f.is_empty(), g.is_empty()) {
        (true, true) => Some((poly![], poly![], poly![])),
        (false, true) => Some((f.clone(), poly![], poly![])),
        (true, false) => Some((g.clone(), poly![], poly![])),
        (false, false) => None,
    }
}

/// Interpolates step-wise gcd of h and x into a polynomial.
fn gcd_interpolate(mut h: isize, x: isize) -> Poly {
    let mut res = Vec::new();
    let mut g: isize;
    while h != 0 {
        g = h % x;
        if g > x / 2 {
            g -= x
        }
        res.push(g);
        h = (h - g) / x;
    }
    Poly::new(res.into_iter().rev().collect())
}

#[cfg(test)]
mod test {
    use super::gcd_poly_zz_heu;
    use crate::math::poly::Poly;
    use crate::poly;

    #[test]
    fn test_gcd_poly_zz_heu_1() {
        assert_eq!(
            gcd_poly_zz_heu(poly![1, 0, -1], poly![1, -3, 2]),
            Ok((poly![1, -1], poly![1, 1], poly![1, -2]))
        )
    }

    #[test]
    fn test_gcd_poly_zz_heu_2() {
        assert_eq!(
            gcd_poly_zz_heu(poly![1, 10, 35, 50, 24], poly![1, 1]),
            Ok((poly![1, 1], poly![1, 9, 26, 24], poly![1]))
        )
    }

    #[test]
    fn test_gcd_poly_zz_heu_3() {
        assert_eq!(
            gcd_poly_zz_heu(poly![1, -9, -1, 9], poly![1, -8, -9]),
            Ok((poly![1, -8, -9], poly![1, -1], poly![1]))
        )
    }
}
