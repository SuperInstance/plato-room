/// Jacobi eigenvalue decomposition for symmetric matrices.
/// Pure Rust, no external dependencies. Converges to 1e-14.

const MAX_ITER: usize = 100;

/// Compute eigenvalues of a symmetric matrix via Jacobi rotations.
/// Returns eigenvalues sorted descending.
pub fn eigenvalues(matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 { return vec![]; }
    let mut a = matrix.to_vec();
    let mut v = identity(n);

    for _ in 0..MAX_ITER * n * n {
        // Find largest off-diagonal element
        let mut max_val = 0.0_f64;
        let mut p = 0_usize;
        let mut q = 1_usize;
        for i in 0..n {
            for j in (i + 1)..n {
                let av = a[i][j].abs();
                if av > max_val {
                    max_val = av;
                    p = i;
                    q = j;
                }
            }
        }
        if max_val < 1e-14 {
            break;
        }

        // Compute rotation angle
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        let theta = if (app - aqq).abs() < 1e-30 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };

        let c = theta.cos();
        let s = theta.sin();

        // Apply rotation to A
        let mut new_a = a.clone();
        new_a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        new_a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        new_a[p][q] = 0.0;
        new_a[q][p] = 0.0;

        for r in 0..n {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                new_a[r][p] = c * arp - s * arq;
                new_a[p][r] = new_a[r][p];
                new_a[r][q] = s * arp + c * arq;
                new_a[q][r] = new_a[r][q];
            }
        }
        a = new_a;

        // Update eigenvector matrix
        for r in 0..n {
            let vrp = v[r][p];
            let vrq = v[r][q];
            v[r][p] = c * vrp - s * vrq;
            v[r][q] = s * vrp + c * vrq;
        }
    }

    let mut eigenvals: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    eigenvals.sort_by(|a, b| b.partial_cmp(a).unwrap());
    eigenvals
}

/// Compute the conservation ratio (λ₁ / Σλᵢ) of a symmetric matrix.
pub fn conservation_ratio(matrix: &[Vec<f64>]) -> f64 {
    let eigs = eigenvalues(matrix);
    if eigs.is_empty() { return 0.0; }
    let sum: f64 = eigs.iter().sum();
    if sum.abs() < 1e-30 { return 0.0; }
    eigs[0] / sum
}

fn identity(n: usize) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n { m[i][i] = 1.0; }
    m
}
