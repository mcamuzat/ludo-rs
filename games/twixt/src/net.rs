//! twixtbot's network ([TB] `torchmodel.py`), BatchNorm folded into the
//! weights by scripts/twixt-export.py. Pure Rust, float32 arithmetic over
//! weights stored as float16.

use std::sync::OnceLock;

use crate::board::SIZE;
use crate::encode::{Inputs, POLICY};

const N: usize = SIZE * SIZE;

/// Raw outputs: policy logits over `POLICY` holes, value logits
/// [loss, draw, win] for the side to move.
#[derive(Clone, Debug)]
pub struct Eval {
    pub policy: Vec<f32>,
    pub value: [f32; 3],
}

struct Conv {
    w: Vec<f32>,
    b: Vec<f32>,
    cout: usize,
    cin: usize,
    k: usize,
}

pub struct Net {
    channels: usize,
    location: Conv,
    pegs: Conv,
    links: Conv,
    blocks: Vec<(Conv, Conv)>,
    policy1: Conv,
    policy2: Conv,
    value: Vec<Conv>,
    fc: (Vec<f32>, Vec<f32>, usize),
    out: Vec<f32>,
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl Reader<'_> {
    fn u32(&mut self) -> Result<u32, String> {
        let (head, rest) = self
            .bytes
            .split_first_chunk::<4>()
            .ok_or("poids tronqués")?;
        self.bytes = rest;
        Ok(u32::from_le_bytes(*head))
    }

    fn f16s(&mut self, n: usize) -> Result<Vec<f32>, String> {
        if self.bytes.len() < 2 * n {
            return Err("poids tronqués".into());
        }
        let (head, rest) = self.bytes.split_at(2 * n);
        self.bytes = rest;
        Ok(head
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&b| f16_to_f32(u16::from_le_bytes(b)))
            .collect())
    }

    fn conv(&mut self, cout: usize, cin: usize, k: usize, bias: bool) -> Result<Conv, String> {
        let w = self.f16s(cout * cin * k * k)?;
        let b = if bias {
            self.f16s(cout)?
        } else {
            vec![0.0; cout]
        };
        Ok(Conv { w, b, cout, cin, k })
    }
}

fn f16_to_f32(h: u16) -> f32 {
    let sign = u32::from(h >> 15) << 31;
    let exp = u32::from((h >> 10) & 0x1f);
    let frac = u32::from(h & 0x3ff);
    let bits = match exp {
        0 if frac == 0 => sign,
        0 => {
            // Subnormal: normalise.
            let mut e = 127 - 15 + 1;
            let mut f = frac;
            while f & 0x400 == 0 {
                f <<= 1;
                e -= 1;
            }
            sign | (e << 23) | ((f & 0x3ff) << 13)
        }
        31 => sign | 0x7f80_0000 | (frac << 13),
        _ => sign | ((exp + 127 - 15) << 23) | (frac << 13),
    };
    f32::from_bits(bits)
}

/// Zero-pads a [c][h][w] tensor.
fn pad(
    x: &[f32],
    c: usize,
    h: usize,
    w: usize,
    (top, bottom): (usize, usize),
    (left, right): (usize, usize),
) -> Vec<f32> {
    let (hp, wp) = (h + top + bottom, w + left + right);
    let mut out = vec![0.0; c * hp * wp];
    for ci in 0..c {
        for r in 0..h {
            let src = &x[(ci * h + r) * w..(ci * h + r + 1) * w];
            let at = (ci * hp + r + top) * wp + left;
            out[at..at + w].copy_from_slice(src);
        }
    }
    out
}

/// Valid convolution of a padded [cin][hp][wp] tensor; adds the bias.
fn conv(x: &[f32], hp: usize, wp: usize, c: &Conv, stride: usize) -> (Vec<f32>, usize, usize) {
    let oh = (hp - c.k) / stride + 1;
    let ow = (wp - c.k) / stride + 1;
    if stride == 1 {
        return (conv_flat(x, hp, wp, c, oh, ow), oh, ow);
    }
    let mut out = vec![0.0; c.cout * oh * ow];
    for co in 0..c.cout {
        let plane = &mut out[co * oh * ow..(co + 1) * oh * ow];
        plane.fill(c.b[co]);
        for ci in 0..c.cin {
            let input = &x[ci * hp * wp..(ci + 1) * hp * wp];
            for ky in 0..c.k {
                for kx in 0..c.k {
                    let wt = c.w[((co * c.cin + ci) * c.k + ky) * c.k + kx];
                    for r in 0..oh {
                        let row = &mut plane[r * ow..(r + 1) * ow];
                        let src_row = (r * stride + ky) * wp;
                        if stride == 1 {
                            let src = &input[src_row + kx..src_row + kx + ow];
                            for (o, &s) in row.iter_mut().zip(src) {
                                *o += wt * s;
                            }
                        } else {
                            for (j, o) in row.iter_mut().enumerate() {
                                *o += wt * input[src_row + j * stride + kx];
                            }
                        }
                    }
                }
            }
        }
    }
    (out, oh, ow)
}

/// Stride-1 convolution on rows flattened at the padded width: each kernel
/// tap is one long multiply-add over the whole plane (the `k − 1` columns
/// past the right edge of every row are computed and dropped).
fn conv_flat(x: &[f32], hp: usize, wp: usize, c: &Conv, oh: usize, ow: usize) -> Vec<f32> {
    let len = (oh - 1) * wp + ow;
    if c.cout.is_multiple_of(CO) {
        return conv_blocked(x, hp, wp, c, oh, ow, len);
    }
    let mut wide = vec![0.0f32; len];
    let mut out = vec![0.0; c.cout * oh * ow];
    for co in 0..c.cout {
        wide.fill(c.b[co]);
        for ci in 0..c.cin {
            let input = &x[ci * hp * wp..(ci + 1) * hp * wp];
            let taps = &c.w[(co * c.cin + ci) * c.k * c.k..(co * c.cin + ci + 1) * c.k * c.k];
            for ky in 0..c.k {
                for kx in 0..c.k {
                    let wt = taps[ky * c.k + kx];
                    let src = &input[ky * wp + kx..ky * wp + kx + len];
                    for (o, &s) in wide.iter_mut().zip(src) {
                        *o += wt * s;
                    }
                }
            }
        }
        let plane = &mut out[co * oh * ow..(co + 1) * oh * ow];
        for r in 0..oh {
            plane[r * ow..(r + 1) * ow].copy_from_slice(&wide[r * wp..r * wp + ow]);
        }
    }
    out
}

/// Output channels and pixels computed together by `conv_blocked`.
const CO: usize = 4;
const PX: usize = 8;

/// `conv_flat` as a small matrix product: `CO` output channels × `PX`
/// pixels accumulate in registers over every (input channel, tap).
fn conv_blocked(
    x: &[f32],
    hp: usize,
    wp: usize,
    c: &Conv,
    oh: usize,
    ow: usize,
    len: usize,
) -> Vec<f32> {
    let k2 = c.k * c.k;
    let chunks = len.div_ceil(PX);
    // Input with room for the last chunk's overrun.
    let mut input = x[..c.cin * hp * wp].to_vec();
    input.resize(c.cin * hp * wp + chunks * PX - len + PX, 0.0);
    let mut out = vec![0.0; c.cout * oh * ow];
    let mut wide = vec![0.0f32; CO * chunks * PX];
    for cob in (0..c.cout).step_by(CO) {
        // Weights of the block, [ci][tap][co].
        let mut wb = vec![0.0f32; c.cin * k2 * CO];
        for j in 0..CO {
            for ci in 0..c.cin {
                for t in 0..k2 {
                    wb[(ci * k2 + t) * CO + j] = c.w[((cob + j) * c.cin + ci) * k2 + t];
                }
            }
        }
        for chunk in 0..chunks {
            let p0 = chunk * PX;
            let mut acc = [[0.0f32; PX]; CO];
            for ci in 0..c.cin {
                let base = ci * hp * wp + p0;
                for ky in 0..c.k {
                    for kx in 0..c.k {
                        let at = base + ky * wp + kx;
                        let s: &[f32; PX] = input[at..at + PX].try_into().expect("PX floats");
                        let w: &[f32; CO] = wb[(ci * k2 + ky * c.k + kx) * CO..][..CO]
                            .try_into()
                            .expect("CO floats");
                        for j in 0..CO {
                            for i in 0..PX {
                                acc[j][i] += w[j] * s[i];
                            }
                        }
                    }
                }
            }
            for j in 0..CO {
                wide[(j * chunks + chunk) * PX..][..PX].copy_from_slice(&acc[j]);
            }
        }
        for j in 0..CO {
            let co = cob + j;
            let src = &wide[j * chunks * PX..];
            let plane = &mut out[co * oh * ow..(co + 1) * oh * ow];
            for r in 0..oh {
                for (o, &s) in plane[r * ow..(r + 1) * ow]
                    .iter_mut()
                    .zip(&src[r * wp..r * wp + ow])
                {
                    *o = s + c.b[co];
                }
            }
        }
    }
    out
}

/// A 5×5 stride-1 convolution keeping 24 × 24 (padding 2).
fn conv_same(x: &[f32], c: &Conv) -> Vec<f32> {
    let p = c.k / 2;
    conv(
        &pad(x, c.cin, SIZE, SIZE, (p, p), (p, p)),
        SIZE + 2 * p,
        SIZE + 2 * p,
        c,
        1,
    )
    .0
}

fn abs_in_place(x: &mut [f32]) {
    x.iter_mut().for_each(|v| *v = v.abs());
}

/// torchmodel.py's SAME padding of a 5×5 stride-2 convolution.
fn same_pad(n: usize) -> (usize, usize) {
    let out = n.div_ceil(2);
    let total = ((out - 1) * 2 + 5).saturating_sub(n);
    (total / 2, total - total / 2)
}

impl Net {
    pub fn from_bytes(bytes: &[u8]) -> Result<Net, String> {
        let magic = bytes.get(..4).ok_or("poids tronqués")?;
        if magic != b"TWXN" {
            return Err("ce fichier n'est pas un réseau TwixT".into());
        }
        let mut r = Reader { bytes: &bytes[4..] };
        if r.u32()? != 1 {
            return Err("version de réseau inconnue".into());
        }
        let c = r.u32()? as usize;
        let blocks = r.u32()? as usize;
        let hidden = r.u32()? as usize;
        let reductions = r.u32()? as usize;
        if c == 0 || c > 512 || blocks > 64 || hidden > 4096 || reductions != 4 {
            return Err("réseau de forme inattendue".into());
        }
        let location = r.conv(c, 2, 1, false)?;
        let pegs = r.conv(c, 2, 5, false)?;
        let mut links = r.conv(c, 8, 4, false)?;
        links.b = r.f16s(c)?; // the stem's single bias
        let blocks = (0..blocks)
            .map(|_| Ok((r.conv(c, c, 5, true)?, r.conv(c, c, 5, true)?)))
            .collect::<Result<Vec<_>, String>>()?;
        let policy1 = r.conv(2, c, 1, true)?;
        let policy2 = r.conv(1, 2, 1, false)?;
        let value = (0..reductions)
            .map(|_| r.conv(c, c, 5, true))
            .collect::<Result<Vec<_>, String>>()?;
        let fc_w = r.f16s(hidden * c * 4)?;
        let fc_b = r.f16s(hidden)?;
        let out = r.f16s(3 * hidden)?;
        if !r.bytes.is_empty() {
            return Err("poids en trop".into());
        }
        Ok(Net {
            channels: c,
            location,
            pegs,
            links,
            blocks,
            policy1,
            policy2,
            value,
            fc: (fc_w, fc_b, hidden),
            out,
        })
    }

    pub fn eval(&self, inputs: &Inputs) -> Eval {
        let c = self.channels;
        // Stem: location 1×1 + pegs 5×5 (pad 2) + links 4×4 (pad 1, 2).
        let mut loc = vec![0.0f32; 2 * N];
        for x in 0..SIZE {
            for y in 0..SIZE {
                loc[x * SIZE + y] = y as f32 / SIZE as f32;
                loc[N + x * SIZE + y] = x as f32 / SIZE as f32;
            }
        }
        let mut h = conv(&loc, SIZE, SIZE, &self.location, 1).0;
        let p = conv_same(&inputs.pegs, &self.pegs);
        let l = conv(
            &pad(&inputs.links, 8, SIZE, SIZE, (1, 2), (1, 2)),
            SIZE + 3,
            SIZE + 3,
            &self.links,
            1,
        )
        .0;
        for ((a, b), d) in h.iter_mut().zip(&p).zip(&l) {
            *a += b + d;
        }
        abs_in_place(&mut h);
        for (c1, c2) in &self.blocks {
            let mut r = conv_same(&h, c1);
            abs_in_place(&mut r);
            let mut r = conv_same(&r, c2);
            abs_in_place(&mut r);
            for (a, b) in h.iter_mut().zip(&r) {
                *a += b;
            }
        }
        // Policy: two 1×1 convolutions, rows x = 1..=22.
        let mut q = conv(&h, SIZE, SIZE, &self.policy1, 1).0;
        abs_in_place(&mut q);
        let q = conv(&q, SIZE, SIZE, &self.policy2, 1).0;
        let policy = q[SIZE..SIZE + POLICY].to_vec();
        // Value: four SAME stride-2 reductions 24 → 12 → 6 → 3 → 2.
        let (mut v, mut n) = (h, SIZE);
        for vc in &self.value {
            let pd = same_pad(n);
            let (mut o, oh, _) = conv(
                &pad(&v, c, n, n, pd, pd),
                n + pd.0 + pd.1,
                n + pd.0 + pd.1,
                vc,
                2,
            );
            abs_in_place(&mut o);
            v = o;
            n = oh;
        }
        // Flatten in TensorFlow's [x][y][c] order.
        let mut flat = vec![0.0f32; c * n * n];
        for ch in 0..c {
            for i in 0..n * n {
                flat[i * c + ch] = v[ch * n * n + i];
            }
        }
        let (fc_w, fc_b, hidden) = &self.fc;
        let hid: Vec<f32> = (0..*hidden)
            .map(|j| {
                let row = &fc_w[j * flat.len()..(j + 1) * flat.len()];
                (fc_b[j] + row.iter().zip(&flat).map(|(a, b)| a * b).sum::<f32>()).abs()
            })
            .collect();
        let value = [0, 1, 2].map(|k| {
            let row = &self.out[k * hidden..(k + 1) * hidden];
            row.iter().zip(&hid).map(|(a, b)| a * b).sum::<f32>()
        });
        Eval { policy, value }
    }
}

static NET: OnceLock<Net> = OnceLock::new();

/// Installs the process-wide network (the browser passes the fetched file).
pub fn install(bytes: &[u8]) -> Result<(), String> {
    if NET.get().is_some() {
        return Ok(());
    }
    let net = Net::from_bytes(bytes)?;
    let _ = NET.set(net);
    Ok(())
}

/// The process-wide network. Natively it is read on first use from
/// `web/src/engine/twixt-net-f16.bin`; in the browser, `None` until `install`.
pub fn net() -> Option<&'static Net> {
    #[cfg(not(target_arch = "wasm32"))]
    if NET.get().is_none() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../web/src/engine/twixt-net-f16.bin"
        );
        if let Ok(bytes) = std::fs::read(path) {
            let _ = install(&bytes);
        }
    }
    NET.get()
}
