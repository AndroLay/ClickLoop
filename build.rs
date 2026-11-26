//! Menanam resource ke exe: ikon (dirender di build/icon.rs), manifest (ComCtl32 v6, DPI per-monitor v2),
//! dan info versi (nama ClickLoop di Task Manager dan Properties).
//! Toolchain gnu: resource dikompilasi dengan windres lalu ditautkan sebagai objek biasa.
//! Semua berkas antara dibuat di OUT_DIR, jadi tidak ada berkas tambahan di proyek.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "build/icon.rs"]
mod icon;

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;

fn rc_path(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/icon.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let ico = out.join("clickloop.ico");
    let manifest = out.join("clickloop.manifest");
    let rc = out.join("clickloop.rc");
    let obj = out.join("clickloop_res.o");

    fs::write(&ico, icon::ico()).expect("gagal menulis ikon");
    fs::write(out.join("icon-256.png"), icon::preview(256)).expect("gagal menulis pratinjau ikon");
    fs::write(&manifest, MANIFEST).expect("gagal menulis manifest");

    let ver = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "1.0.0".into());
    let mut nums: Vec<u32> = ver.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    nums.resize(4, 0);
    let commas = format!("{},{},{},{}", nums[0], nums[1], nums[2], nums[3]);

    let rc_text = format!(
        r#"1 ICON "{ico}"
1 24 "{manifest}"
1 VERSIONINFO
FILEVERSION {commas}
PRODUCTVERSION {commas}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "FileDescription", "ClickLoop"
      VALUE "FileVersion", "{ver}"
      VALUE "InternalName", "clickloop"
      VALUE "OriginalFilename", "clickloop.exe"
      VALUE "ProductName", "ClickLoop"
      VALUE "ProductVersion", "{ver}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        ico = rc_path(&ico),
        manifest = rc_path(&manifest),
    );
    fs::write(&rc, rc_text).expect("gagal menulis berkas rc");

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let status = Command::new("windres")
        .arg(&rc)
        .arg("-O")
        .arg("coff")
        .arg("-o")
        .arg(&obj)
        .status()
        .unwrap_or_else(|e| {
            if target_env == "msvc" {
                panic!(
                    "windres tidak ditemukan di PATH ({e}). Proyek ini menggunakan target x86_64-pc-windows-gnu dengan windres (MinGW-w64 / MSYS2 UCRT64); lihat README.md."
                );
            }
            panic!("windres tidak ditemukan di PATH: {e}");
        });
    assert!(status.success(), "windres gagal memproses resource");
    println!("cargo:rustc-link-arg-bins={}", obj.display());
}
