//! Ikon ClickLoop dirender saat build, tanpa berkas gambar dan tanpa crate:
//! kotak membulat biru bergradasi, cincin loop putih dengan kepala panah, dan kursor putih di tengah dengan bayangan lembut.
//! Hasilnya .ico multi-ukuran: BMP 32-bit untuk ukuran kecil, PNG (deflate sendiri) untuk 256 px.

const SIZES: [u32; 8] = [16, 20, 24, 32, 40, 48, 64, 256];

/// Panah kursor klasik dalam satuan "piksel panah", ujung di (0, 0).
const ARROW: [(f32, f32); 7] = [
    (0.0, 0.0),
    (0.0, 15.2),
    (3.7, 11.7),
    (6.3, 17.6),
    (8.9, 16.5),
    (6.3, 10.7),
    (11.2, 10.7),
];
const TIP: (f32, f32) = (0.42, 0.38);
const SCALE: f32 = 0.017;
/// Cincin "loop": busur berpusat di tengah, terbuka di kanan-atas, ujungnya berkepala panah.
const RING_C: (f32, f32) = (0.5, 0.5);
const RING_R: f32 = 0.30;
/// Setengah tebal cincin minimum; di ukuran kecil dipertebal lewat `lw`.
const RING_HALF: f32 = 0.024;
/// Awal busur (radian, -20 derajat) dan panjangnya (~280 derajat), searah jarum jam di layar.
const RING_START: f32 = -0.35;
const RING_SPAN: f32 = 4.9;

pub fn ico() -> Vec<u8> {
    let images: Vec<Vec<u8>> = SIZES
        .iter()
        .map(|&s| {
            let px = render(s);
            if s >= 256 {
                png(s, &px)
            } else {
                dib(s, &px)
            }
        })
        .collect();
    let mut out = Vec::new();
    put16(&mut out, 0);
    put16(&mut out, 1);
    put16(&mut out, SIZES.len() as u16);
    let mut offset = 6 + 16 * SIZES.len() as u32;
    for (s, img) in SIZES.iter().zip(&images) {
        let b = if *s >= 256 { 0 } else { *s as u8 };
        out.extend([b, b, 0, 0]);
        put16(&mut out, 1);
        put16(&mut out, 32);
        put32(&mut out, img.len() as u32);
        put32(&mut out, offset);
        offset += img.len() as u32;
    }
    for img in images {
        out.extend(img);
    }
    out
}

/// PNG satu ukuran, untuk pratinjau saat pengembangan.
pub fn preview(size: u32) -> Vec<u8> {
    png(size, &render(size))
}

/// RGBA (alpha lurus), dengan supersampling per piksel.
fn render(size: u32) -> Vec<u8> {
    let n = size as usize;
    let ss = if size <= 64 { 4 } else { 3 };
    let lw = RING_HALF.max(0.8 / size as f32);
    let mut out = vec![0u8; n * n * 4];
    for py in 0..n {
        for px in 0..n {
            let mut acc = [0.0f32; 3];
            let mut hits = 0.0f32;
            for sy in 0..ss {
                for sx in 0..ss {
                    let x = (px as f32 + (sx as f32 + 0.5) / ss as f32) / size as f32;
                    let y = (py as f32 + (sy as f32 + 0.5) / ss as f32) / size as f32;
                    if let Some(c) = sample(x, y, lw) {
                        acc[0] += c[0];
                        acc[1] += c[1];
                        acc[2] += c[2];
                        hits += 1.0;
                    }
                }
            }
            if hits > 0.0 {
                let i = (py * n + px) * 4;
                out[i] = to8(acc[0] / hits);
                out[i + 1] = to8(acc[1] / hits);
                out[i + 2] = to8(acc[2] / hits);
                out[i + 3] = to8(hits / (ss * ss) as f32);
            }
        }
    }
    out
}

fn sample(x: f32, y: f32, lw: f32) -> Option<[f32; 3]> {
    if round_box(x, y) > 0.0 {
        return None;
    }
    let t = ((x + y) * 0.5).clamp(0.0, 1.0);
    let top = [0.29, 0.66, 1.0];
    let bottom = [0.0, 0.33, 0.78];
    let mut c = [
        top[0] + (bottom[0] - top[0]) * t,
        top[1] + (bottom[1] - top[1]) * t,
        top[2] + (bottom[2] - top[2]) * t,
    ];
    // Bayangan lembut di kanan-bawah panah.
    let sd = arrow_sd(x - 0.018, y - 0.03);
    let k = ((0.05 - sd) / 0.07).clamp(0.0, 1.0);
    let shade = 1.0 - 0.32 * k * k * (3.0 - 2.0 * k);
    for v in c.iter_mut() {
        *v *= shade;
    }
    if ring_hit(x, y, lw) || arrow_sd(x, y) <= 0.0 {
        c = [1.0, 1.0, 1.0];
    }
    Some(c)
}

/// Jarak bertanda ke kotak membulat (margin 1/16, radius 0,2).
fn round_box(x: f32, y: f32) -> f32 {
    let r = 0.2;
    let half = 0.5 - 0.0625;
    let qx = (x - 0.5).abs() - (half - r);
    let qy = (y - 0.5).abs() - (half - r);
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

fn arrow_pt(i: usize) -> (f32, f32) {
    let (ax, ay) = ARROW[i % ARROW.len()];
    (TIP.0 + ax * SCALE, TIP.1 + ay * SCALE)
}

/// Jarak bertanda ke poligon (negatif di dalam).
fn poly_sd<const N: usize>(x: f32, y: f32, pts: [(f32, f32); N]) -> f32 {
    let mut d = f32::MAX;
    let mut inside = false;
    for i in 0..N {
        let a = pts[i];
        let b = pts[(i + 1) % N];
        d = d.min(seg_dist(x, y, a, b));
        if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
            inside = !inside;
        }
    }
    if inside {
        -d
    } else {
        d
    }
}

/// Jarak bertanda ke poligon panah kursor (negatif di dalam).
fn arrow_sd(x: f32, y: f32) -> f32 {
    let pts: [(f32, f32); 7] = std::array::from_fn(arrow_pt);
    poly_sd(x, y, pts)
}

/// Titik pada lingkaran cincin: sudut `a` (radian) dan jari-jari `r`.
fn ring_pt(a: f32, r: f32) -> (f32, f32) {
    (RING_C.0 + r * a.cos(), RING_C.1 + r * a.sin())
}

/// Cincin loop beserta kepala panahnya di ujung busur.
fn ring_hit(x: f32, y: f32, lw: f32) -> bool {
    let (dx, dy) = (x - RING_C.0, y - RING_C.1);
    let r = (dx * dx + dy * dy).sqrt();
    if (r - RING_R).abs() <= lw {
        let t = (dy.atan2(dx) - RING_START).rem_euclid(std::f32::consts::TAU);
        if t <= RING_SPAN {
            return true;
        }
    }
    let end = RING_START + RING_SPAN;
    let head = [
        ring_pt(end, RING_R - 0.075),
        ring_pt(end, RING_R + 0.075),
        ring_pt(end + 0.32, RING_R),
    ];
    poly_sd(x, y, head) <= 0.0
}

fn seg_dist(x: f32, y: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let (wx, wy) = (x - a.0, y - a.1);
    let t = ((wx * vx + wy * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    let (dx, dy) = (wx - vx * t, wy - vy * t);
    (dx * dx + dy * dy).sqrt()
}

fn to8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

fn put16(v: &mut Vec<u8>, x: u16) {
    v.extend(x.to_le_bytes());
}

fn put32(v: &mut Vec<u8>, x: u32) {
    v.extend(x.to_le_bytes());
}

/// Entri ICO format DIB: BITMAPINFOHEADER, piksel BGRA dari bawah ke atas, lalu mask AND.
fn dib(size: u32, rgba: &[u8]) -> Vec<u8> {
    let n = size as usize;
    let stride = n.div_ceil(32) * 4;
    let mut v = Vec::with_capacity(40 + n * n * 4 + stride * n);
    put32(&mut v, 40);
    put32(&mut v, size);
    put32(&mut v, size * 2);
    put16(&mut v, 1);
    put16(&mut v, 32);
    put32(&mut v, 0);
    put32(&mut v, (n * n * 4 + stride * n) as u32);
    for _ in 0..4 {
        put32(&mut v, 0);
    }
    for row in (0..n).rev() {
        for col in 0..n {
            let i = (row * n + col) * 4;
            v.extend([rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    for row in (0..n).rev() {
        let mut line = vec![0u8; stride];
        for col in 0..n {
            if rgba[(row * n + col) * 4 + 3] == 0 {
                line[col / 8] |= 0x80 >> (col % 8);
            }
        }
        v.extend(line);
    }
    v
}

fn png(size: u32, rgba: &[u8]) -> Vec<u8> {
    let n = size as usize;
    let mut raw = Vec::with_capacity(n * (n * 4 + 1));
    for row in 0..n {
        raw.push(1); // filter Sub
        let line = &rgba[row * n * 4..(row + 1) * n * 4];
        for i in 0..line.len() {
            let left = if i >= 4 { line[i - 4] } else { 0 };
            raw.push(line[i].wrapping_sub(left));
        }
    }
    let mut z = vec![0x78, 0x01];
    z.extend(deflate(&raw));
    z.extend(adler32(&raw).to_be_bytes());

    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend(size.to_be_bytes());
    ihdr.extend(size.to_be_bytes());
    ihdr.extend([8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, ty: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(ty);
    out.extend(data);
    let crc = crc32(&out[start..]);
    out.extend(crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Penulis bit deflate (LSB dulu).
struct Bits {
    out: Vec<u8>,
    acc: u32,
    n: u32,
}

impl Bits {
    fn put(&mut self, v: u32, len: u32) {
        self.acc |= v << self.n;
        self.n += len;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// Kode Huffman ditulis MSB dulu.
    fn code(&mut self, code: u32, len: u32) {
        let mut r = 0;
        for i in 0..len {
            r |= ((code >> i) & 1) << (len - 1 - i);
        }
        self.put(r, len);
    }

    fn literal(&mut self, v: u32) {
        match v {
            0..=143 => self.code(0x30 + v, 8),
            144..=255 => self.code(0x190 + v - 144, 9),
            256..=279 => self.code(v - 256, 7),
            _ => self.code(0xC0 + v - 280, 8),
        }
    }
}

const LBASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEXT: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DBASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DEXT: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

const WIN: usize = 32768;
const HBITS: usize = 15;
const NONE: usize = usize::MAX;

/// Indeks LZ77: rantai hash 3 byte.
struct Chains<'a> {
    data: &'a [u8],
    head: Vec<usize>,
    prev: Vec<usize>,
}

impl Chains<'_> {
    fn hash(&self, i: usize) -> usize {
        let d = self.data;
        ((d[i] as usize) << 10 ^ (d[i + 1] as usize) << 5 ^ d[i + 2] as usize) & ((1 << HBITS) - 1)
    }

    fn insert(&mut self, i: usize) {
        if i + 2 < self.data.len() {
            let h = self.hash(i);
            self.prev[i % WIN] = self.head[h];
            self.head[h] = i;
        }
    }

    /// Kecocokan terpanjang (panjang, jarak) untuk posisi i.
    fn best(&self, i: usize) -> (usize, usize) {
        let d = self.data;
        if i + 2 >= d.len() {
            return (0, 0);
        }
        let max = (d.len() - i).min(258);
        let (mut best_len, mut best_dist) = (0, 0);
        let mut cand = self.head[self.hash(i)];
        let mut chain = 0;
        while cand != NONE && cand < i && i - cand <= WIN && chain < 128 {
            let mut l = 0;
            while l < max && d[cand + l] == d[i + l] {
                l += 1;
            }
            if l > best_len {
                best_len = l;
                best_dist = i - cand;
                if l == max {
                    break;
                }
            }
            let next = self.prev[cand % WIN];
            if next == NONE || next >= cand {
                break;
            }
            cand = next;
            chain += 1;
        }
        (best_len, best_dist)
    }
}

/// Deflate satu blok Huffman tetap dengan LZ77 rantai hash. Cukup untuk ikon yang datar.
fn deflate(data: &[u8]) -> Vec<u8> {
    let mut b = Bits {
        out: Vec::new(),
        acc: 0,
        n: 0,
    };
    b.put(1, 1); // blok terakhir
    b.put(1, 2); // Huffman tetap
    let mut ch = Chains {
        data,
        head: vec![NONE; 1 << HBITS],
        prev: vec![NONE; WIN],
    };
    let mut i = 0;
    while i < data.len() {
        let (len, dist) = ch.best(i);
        if len >= 3 {
            let li = (0..29).rev().find(|&k| LBASE[k] <= len as u32).unwrap_or(0);
            b.literal(257 + li as u32);
            b.put(len as u32 - LBASE[li], LEXT[li]);
            let di = (0..30)
                .rev()
                .find(|&k| DBASE[k] <= dist as u32)
                .unwrap_or(0);
            b.code(di as u32, 5);
            b.put(dist as u32 - DBASE[di], DEXT[di]);
            for k in 0..len {
                ch.insert(i + k);
            }
            i += len;
        } else {
            b.literal(data[i] as u32);
            ch.insert(i);
            i += 1;
        }
    }
    b.literal(256);
    if b.n > 0 {
        b.out.push(b.acc as u8);
    }
    b.out
}
