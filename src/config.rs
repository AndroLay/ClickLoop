//! Pengaturan ClickLoop. Disimpan sebagai key=value di clickloop.ini, di samping exe.
//! Bila berkas itu belum ada, auto-clicker.ini (nama lama) dibaca agar pengaturan terbawa.
//! Profil memakai format yang sama dan disimpan lewat dialog sebagai berkas .ini.
use std::fs;
use std::path::{Path, PathBuf};

const FILE: &str = "clickloop.ini";
const LEGACY_FILE: &str = "auto-clicker.ini";

pub const MIN_POINTS: usize = 2;
pub const MAX_POINTS: usize = 5;
/// Batas atas delayed start: 99 menit + 99 detik + 9999 ms.
pub const MAX_DELAY_MS: u32 = 99 * 60_000 + 99 * 1000 + 9999;
pub const MAX_POS_JITTER: u32 = 200;
pub const MAX_MILLIS: u32 = 9999;
/// Interval total di bawah batas ini (ms) dianggap sangat cepat dan memicu peringatan di UI.
pub const LOW_INTERVAL_MS: u64 = 10;
/// 99 jam 59 menit 59,999 detik, batas opsi baris perintah --interval.
const MAX_INTERVAL_MS: u64 = 359_999_999;

/// Kombinasi hotkey: virtual-key + modifier gaya HOTKEYF (Shift 1, Ctrl 2, Alt 4).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub struct Hotkey {
    pub vk: u32,
    pub mods: u32,
}

impl Hotkey {
    pub const fn plain(vk: u32) -> Self {
        Hotkey { vk, mods: 0 }
    }
}

#[derive(Clone)]
pub struct Settings {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub millis: u32,
    /// Interval ditambah/dikurangi acak sampai nilai ini (ms).
    pub jitter: u32,
    /// 0 = kiri, 1 = kanan, 2 = tengah.
    pub button: u32,
    /// 0 = single, 1 = double, 2 = triple.
    pub kind: u32,
    /// Lama tombol ditahan per klik (ms). 0 = klik cepat.
    pub press_ms: u32,
    /// Berhenti setelah sekian klik. 0 = tanpa batas.
    pub max_clicks: u32,
    /// Berhenti setelah sekian detik. 0 = tanpa batas.
    pub max_secs: u32,
    pub fixed: bool,
    /// Tombol mouse ditekan terus sampai berhenti; interval dan jumlah klik diabaikan.
    pub hold: bool,
    pub x: i32,
    pub y: i32,
    /// Acak posisi klik (px) per sumbu. Berlaku bila posisi tetap atau sequence.
    pub pos_jitter: u32,
    /// Sequence: klik berpindah berurutan ke setiap titik.
    pub sequence: bool,
    /// Titik sequence, MIN_POINTS..=MAX_POINTS.
    pub points: Vec<(i32, i32)>,
    /// Jeda sebelum mulai (milidetik). 0 = langsung.
    pub delay_ms: u32,
    /// Putar makro berulang terus sampai dihentikan.
    pub macro_loop: bool,
    /// Jeda otomatis saat jendela target kehilangan fokus.
    pub focus_guard: bool,
    /// Berhenti saat kursor digoyang cepat atau berada di pojok kiri-atas layar.
    pub failsafe: bool,
    /// Trigger tahan: klik hanya berjalan selama tombol start ditahan.
    pub trigger_hold: bool,
    /// Tombol penahan untuk mode hold: 0 = Mouse4, 1 = Mouse5.
    pub trigger_btn: u32,
    /// Saat diminimalkan, jendela masuk ke tray.
    pub tray: bool,
    pub start_key: Hotkey,
    pub pick_key: Hotkey,
    pub rec_key: Hotkey,
    pub play_key: Hotkey,
    pub topmost: bool,
    /// Mode gelap; hanya disimpan di pengaturan utama.
    pub dark: bool,
    /// Posisi kiri-atas jendela terakhir (piksel fisik).
    pub win_pos: Option<(i32, i32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hours: 0,
            minutes: 0,
            seconds: 0,
            millis: 100,
            jitter: 0,
            button: 0,
            kind: 0,
            press_ms: 0,
            max_clicks: 0,
            max_secs: 0,
            fixed: false,
            hold: false,
            x: 0,
            y: 0,
            pos_jitter: 0,
            sequence: false,
            points: Vec::new(),
            delay_ms: 0,
            macro_loop: false,
            focus_guard: false,
            failsafe: false,
            trigger_hold: false,
            trigger_btn: 0,
            tray: false,
            start_key: Hotkey::plain(0x75), // F6
            pick_key: Hotkey::plain(0x77),  // F8
            rec_key: Hotkey::plain(0x78),   // F9
            play_key: Hotkey::plain(0x79),  // F10
            topmost: false,
            dark: false,
            win_pos: None,
        }
    }
}

impl Settings {
    /// Interval dasar dalam milidetik.
    pub fn interval_ms(&self) -> u64 {
        let s = self.hours as u64 * 3600 + self.minutes as u64 * 60 + self.seconds as u64;
        s * 1000 + self.millis as u64
    }

    /// Jumlah klik per aksi: 1, 2, atau 3.
    pub fn clicks_per_action(&self) -> u32 {
        self.kind + 1
    }

    /// Memecah `delay_ms` menjadi `(menit, detik, ms)` untuk kotak isian Delayed start.
    pub fn delay_parts(&self) -> (u32, u32, u32) {
        let mins = (self.delay_ms / 60_000).min(99);
        let rem = self.delay_ms.saturating_sub(mins * 60_000);
        let secs = (rem / 1000).min(99);
        let ms = rem.saturating_sub(secs * 1000).min(MAX_MILLIS);
        (mins, secs, ms)
    }

    /// Titik klik yang dipaksa (sequence atau posisi tetap). None = klik di posisi kursor saat ini.
    pub fn aim_points(&self) -> Option<Vec<(i32, i32)>> {
        if self.sequence && self.points.len() >= MIN_POINTS {
            Some(self.points.clone())
        } else if self.fixed {
            Some(vec![(self.x, self.y)])
        } else {
            None
        }
    }
}

fn dir_file(name: &str) -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn path() -> PathBuf {
    dir_file(FILE)
}

fn uint(v: &str, def: u32, lo: u32, hi: u32) -> u32 {
    v.parse::<u64>()
        .map(|n| n.clamp(lo as u64, hi as u64) as u32)
        .unwrap_or(def)
}

fn flag(v: &str) -> bool {
    v == "1" || v.eq_ignore_ascii_case("true")
}

fn pair(v: &str) -> Option<(i32, i32)> {
    let (a, b) = v.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// "x,y;x,y;..." menjadi daftar titik. Titik rusak dilewati, paling banyak MAX_POINTS.
fn points(v: &str) -> Vec<(i32, i32)> {
    v.split(';').filter_map(pair).take(MAX_POINTS).collect()
}

/// "vk,mods" atau "vk" saja (tanpa modifier). Hanya kombinasi sah (sesuai `keys::check`) yang diterima.
fn hotkey(v: &str, def: Hotkey) -> Hotkey {
    let (vk, mods) = match v.split_once(',') {
        Some((a, b)) => (a.trim().parse::<i64>().ok(), b.trim().parse::<i64>().ok()),
        None => (v.parse::<i64>().ok(), Some(0)),
    };
    match (vk, mods) {
        (Some(vk), Some(mods)) if (1..=254).contains(&vk) => {
            let h = Hotkey {
                vk: vk as u32,
                mods: (mods as u32) & 7,
            };
            if crate::keys::check(h).is_ok() {
                h
            } else {
                def
            }
        }
        _ => def,
    }
}

/// Nomor tombol F lama (1..=12) menjadi hotkey.
fn fkey(v: &str, def: Hotkey) -> Hotkey {
    match v.parse::<u32>() {
        Ok(n @ 1..=12) => Hotkey::plain(0x70 + n - 1),
        _ => def,
    }
}

/// Membaca teks pengaturan. Baris rusak dilewati; nilai di-clamp.
fn parse(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim();
        match k.trim() {
            "hours" | "jam" => s.hours = uint(v, s.hours, 0, 99),
            "minutes" | "menit" => s.minutes = uint(v, s.minutes, 0, 99),
            "seconds" | "detik" => s.seconds = uint(v, s.seconds, 0, 99),
            "millis" | "ms" => s.millis = uint(v, s.millis, 0, MAX_MILLIS),
            "jitter" | "acak" => s.jitter = uint(v, s.jitter, 0, 9999),
            "button" | "tombol" => s.button = uint(v, s.button, 0, 2),
            "click_type" | "tipe" => s.kind = uint(v, s.kind, 0, 2),
            "press_ms" | "tahan" => s.press_ms = uint(v, s.press_ms, 0, 60_000),
            "max_clicks" | "batas_klik" => s.max_clicks = uint(v, s.max_clicks, 0, u32::MAX),
            "max_seconds" | "batas_detik" => s.max_secs = uint(v, s.max_secs, 0, u32::MAX),
            "fixed_position" | "posisi_tetap" => s.fixed = flag(v),
            "hold" => s.hold = flag(v),
            "x" => s.x = v.parse().unwrap_or(s.x),
            "y" => s.y = v.parse().unwrap_or(s.y),
            "pos_jitter" => s.pos_jitter = uint(v, s.pos_jitter, 0, MAX_POS_JITTER),
            "sequence" => s.sequence = flag(v),
            "points" => s.points = points(v),
            "delay" => s.delay_ms = uint(v, 0, 0, MAX_DELAY_MS / 1000).saturating_mul(1000),
            "delay_ms" => s.delay_ms = uint(v, s.delay_ms, 0, MAX_DELAY_MS),
            "macro_loop" => s.macro_loop = flag(v),
            "focus_guard" => s.focus_guard = flag(v),
            "failsafe" => s.failsafe = flag(v),
            "trigger_hold" => s.trigger_hold = flag(v),
            "trigger_btn" => s.trigger_btn = uint(v, s.trigger_btn, 0, 1),
            "tray" => s.tray = flag(v),
            "start_key" => s.start_key = hotkey(v, s.start_key),
            "pick_key" => s.pick_key = hotkey(v, s.pick_key),
            "rec_key" => s.rec_key = hotkey(v, s.rec_key),
            "play_key" => s.play_key = hotkey(v, s.play_key),
            "hk_mulai" => s.start_key = fkey(v, s.start_key),
            "hk_posisi" => s.pick_key = fkey(v, s.pick_key),
            "topmost" => s.topmost = flag(v),
            "dark" => s.dark = flag(v),
            "window" => s.win_pos = pair(v),
            _ => {}
        }
    }
    let keys_all = [s.start_key, s.pick_key, s.rec_key, s.play_key];
    let has_dup = (0..keys_all.len()).any(|i| keys_all[i + 1..].contains(&keys_all[i]));
    if has_dup {
        let d = Settings::default();
        s.start_key = d.start_key;
        s.pick_key = d.pick_key;
        s.rec_key = d.rec_key;
        s.play_key = d.play_key;
    }
    s
}

/// Membaca pengaturan utama. Berkas hilang atau nilai rusak memakai default.
pub fn load() -> Settings {
    fs::read_to_string(path())
        .or_else(|_| fs::read_to_string(dir_file(LEGACY_FILE)))
        .map(|t| parse(&t))
        .unwrap_or_default()
}

/// Membaca profil dari berkas tertentu. None bila berkas tidak bisa dibaca.
pub fn read_file(p: &Path) -> Option<Settings> {
    fs::read_to_string(p).ok().map(|t| parse(&t))
}

/// Teks pengaturan. Posisi jendela hanya disertakan untuk berkas utama, agar profil tidak memindahkan jendela.
fn to_text(s: &Settings, window: bool) -> String {
    let pts = s
        .points
        .iter()
        .map(|(x, y)| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(";");
    let mut text = format!(
        "hours={}\nminutes={}\nseconds={}\nmillis={}\njitter={}\nbutton={}\nclick_type={}\n\
         press_ms={}\nmax_clicks={}\nmax_seconds={}\nfixed_position={}\nhold={}\nx={}\ny={}\n\
         pos_jitter={}\nsequence={}\npoints={}\ndelay_ms={}\nmacro_loop={}\nfocus_guard={}\nfailsafe={}\n\
         trigger_hold={}\ntrigger_btn={}\ntray={}\nstart_key={},{}\npick_key={},{}\nrec_key={},{}\nplay_key={},{}\ntopmost={}\n",
        s.hours,
        s.minutes,
        s.seconds,
        s.millis,
        s.jitter,
        s.button,
        s.kind,
        s.press_ms,
        s.max_clicks,
        s.max_secs,
        s.fixed as u32,
        s.hold as u32,
        s.x,
        s.y,
        s.pos_jitter,
        s.sequence as u32,
        pts,
        s.delay_ms,
        s.macro_loop as u32,
        s.focus_guard as u32,
        s.failsafe as u32,
        s.trigger_hold as u32,
        s.trigger_btn,
        s.tray as u32,
        s.start_key.vk,
        s.start_key.mods,
        s.pick_key.vk,
        s.pick_key.mods,
        s.rec_key.vk,
        s.rec_key.mods,
        s.play_key.vk,
        s.play_key.mods,
        s.topmost as u32,
    );
    if window {
        // Tema hanya untuk pengaturan utama, bukan profil.
        text.push_str(&format!("dark={}\n", s.dark as u32));
        if let Some((x, y)) = s.win_pos {
            text.push_str(&format!("window={x},{y}\n"));
        }
    }
    text
}

pub fn save(s: &Settings) -> std::io::Result<()> {
    fs::write(path(), to_text(s, true))
}

/// Menyimpan profil ke berkas yang dipilih pengguna.
pub fn save_file(p: &Path, s: &Settings) -> std::io::Result<()> {
    fs::write(p, to_text(s, false))
}

fn num(v: &str, name: &str) -> Result<u64, String> {
    v.parse::<u64>()
        .map_err(|_| format!("{name} needs a whole number, got \"{v}\"."))
}

/// Opsi baris perintah. --profile dimuat dulu, lalu opsi lain menimpanya.
/// Mengembalikan true bila --start diminta. Galat berupa pesan untuk ditampilkan.
pub fn apply_args(s: &mut Settings, args: &[String]) -> Result<bool, String> {
    // Temukan --profile (mengabaikan nilai milik opsi lain) agar dimuat sebelum opsi lain menimpanya.
    let mut profile_path: Option<&str> = None;
    let mut j = 0;
    while j < args.len() {
        match args[j].as_str() {
            "--profile" => {
                let p = args.get(j + 1).ok_or("--profile needs a file path.")?;
                profile_path = Some(p.as_str());
                j += 2;
            }
            "--interval" | "--limit" | "--time" | "--jitter" | "--button" => j += 2,
            _ => j += 1,
        }
    }
    if let Some(path) = profile_path {
        let win = s.win_pos;
        let dark = s.dark;
        *s = read_file(Path::new(path)).ok_or_else(|| format!("Cannot read profile: {path}"))?;
        s.win_pos = win;
        s.dark = dark;
    }
    let mut start = false;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        i += 1;
        match a {
            "--start" => start = true,
            "--profile" => i += 1,
            "--interval" | "--limit" | "--time" | "--jitter" | "--button" => {
                let v = args.get(i).ok_or_else(|| format!("{a} needs a value."))?;
                i += 1;
                match a {
                    "--interval" => {
                        let ms = num(v, a)?.min(MAX_INTERVAL_MS);
                        s.hours = (ms / 3_600_000) as u32;
                        s.minutes = (ms / 60_000 % 60) as u32;
                        s.seconds = (ms / 1000 % 60) as u32;
                        s.millis = (ms % 1000) as u32;
                    }
                    "--limit" => s.max_clicks = num(v, a)?.min(u32::MAX as u64) as u32,
                    "--time" => s.max_secs = num(v, a)?.min(u32::MAX as u64) as u32,
                    "--jitter" => s.jitter = num(v, a)?.min(9999) as u32,
                    _ => {
                        s.button = match v.to_ascii_lowercase().as_str() {
                            "left" => 0,
                            "right" => 1,
                            "middle" => 2,
                            _ => {
                                return Err(format!(
                                    "--button must be left, right, or middle, got \"{v}\"."
                                ))
                            }
                        }
                    }
                }
            }
            _ => return Err(format!("Unknown option: {a}")),
        }
    }
    Ok(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clamps_validates_hotkeys_and_handles_legacy_keys() {
        let raw = "\
jam=150
menit=120
detik=105
ms=15000
acak=99999
tombol=5
tipe=2
tahan=999999
batas_klik=42
batas_detik=10
posisi_tetap=true
x=-120
y=340
pos_jitter=999
sequence=1
points=1,2;bad;3,4;5,6;7,8;9,10;11,12
delay=99
start_key=65,0
pick_key=75,3
topmost=1
dark=1
window=100,200
";
        let s = parse(raw);
        assert_eq!(s.hours, 99);
        assert_eq!(s.minutes, 99);
        assert_eq!(s.seconds, 99);
        assert_eq!(s.millis, MAX_MILLIS);
        assert_eq!(parse("ms=2500\n").millis, 2500);
        assert_eq!(s.jitter, 9999);
        assert_eq!(s.button, 2);
        assert_eq!(s.kind, 2);
        assert_eq!(s.press_ms, 60_000);
        assert_eq!(s.max_clicks, 42);
        assert_eq!(s.max_secs, 10);
        assert!(s.fixed);
        assert_eq!((s.x, s.y), (-120, 340));
        assert_eq!(s.pos_jitter, MAX_POS_JITTER);
        assert!(s.sequence);
        assert_eq!(s.points, vec![(1, 2), (3, 4), (5, 6), (7, 8), (9, 10)]);
        assert_eq!(s.delay_ms, 99_000);
        assert_eq!(
            parse("delay_ms=1500\nmacro_loop=1\n").delay_parts(),
            (0, 1, 500)
        );
        assert!(parse("macro_loop=1\n").macro_loop);
        // 'A' tanpa Ctrl/Alt harus ditolak dan tetap memakai default F6 (0x75).
        assert_eq!(s.start_key, Hotkey::plain(0x75));
        // Ctrl+Shift+K (mods = 3) sah.
        assert_eq!(s.pick_key, Hotkey { vk: 75, mods: 3 });
        assert!(s.topmost);
        assert!(s.dark);
        assert_eq!(s.win_pos, Some((100, 200)));
    }

    #[test]
    fn parse_resets_duplicate_hotkeys_to_defaults() {
        let s = parse("start_key=118,0\npick_key=118,0\n");
        let d = Settings::default();
        assert_eq!(s.start_key, d.start_key);
        assert_eq!(s.pick_key, d.pick_key);
        assert_eq!(s.rec_key, d.rec_key);
        assert_eq!(s.play_key, d.play_key);

        let s2 = parse("rec_key=120,0\nplay_key=120,0\n");
        assert_eq!(s2.rec_key, d.rec_key);
        assert_eq!(s2.play_key, d.play_key);

        let s3 = parse("rec_key=121,0\nplay_key=122,0\n");
        assert_eq!(s3.rec_key, Hotkey::plain(121));
        assert_eq!(s3.play_key, Hotkey::plain(122));
    }

    #[test]
    fn apply_args_profile_preserves_dark_and_win_pos_and_allows_overrides() {
        let dir = std::env::temp_dir();
        let prof = dir.join(format!("clickloop_test_profile_{}.ini", std::process::id()));
        let p = Settings {
            millis: 250,
            button: 1,
            topmost: true,
            ..Default::default()
        };
        save_file(&prof, &p).unwrap();

        let mut s = Settings {
            dark: true,
            win_pos: Some((50, 60)),
            ..Default::default()
        };
        let args: Vec<String> = [
            "--interval",
            "65432",
            "--profile",
            prof.to_str().unwrap(),
            "--limit",
            "500",
            "--button",
            "middle",
            "--start",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();

        let start = apply_args(&mut s, &args).unwrap();
        let _ = fs::remove_file(&prof);

        assert!(start);
        assert!(s.dark);
        assert_eq!(s.win_pos, Some((50, 60)));
        assert!(s.topmost);
        assert_eq!(s.interval_ms(), 65_432);
        assert_eq!(s.max_clicks, 500);
        assert_eq!(s.button, 2);
    }

    #[test]
    fn apply_args_rejects_invalid_options() {
        let mut s = Settings::default();
        let mut bad =
            |a: &[&str]| apply_args(&mut s, &a.iter().map(|x| x.to_string()).collect::<Vec<_>>());
        assert!(bad(&["--unknown"]).is_err());
        assert!(bad(&["--interval"]).is_err());
        assert!(bad(&["--interval", "abc"]).is_err());
        assert!(bad(&["--button", "xbutton"]).is_err());
        assert!(bad(&["--profile"]).is_err());
    }
}
