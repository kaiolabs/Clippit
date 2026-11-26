{
  lib,
  rustPlatform,
  pkg-config,
  makeWrapper,
  wrapGAppsHook4,
  gtk4,
  libadwaita,
  glib,
  gobject-introspection,
  ibus,
  tesseract,
  leptonica,
  libx11,
  libxcb,
  libxkbcommon,
  wayland,
  hunspell,
  hunspellDicts,
  symlinkJoin,
  python3,
  wl-clipboard,
  imagemagick,
  poppler-utils,
  ghostscript,
  playerctl,
  libnotify,
  curl,
  sqlite,
  libarchive,
  quickshell,
  wtype,
  grim,
  pipewire,
  fd,
  cairo,
  pango,
  llama-cpp,
  # The local AI server. Pass `withAI = false` to leave it out, or a GPU build
  # with `llama-cpp = pkgs.llama-cpp.override { vulkanSupport = true; }`.
  withAI ? true,
}:

let
  cargoToml = lib.importTOML ../Cargo.toml;

  ocr = tesseract.override {
    enableLanguages = [
      "eng"
      "por"
    ];
  };

  dictionaries = symlinkJoin {
    name = "clippit-hunspell-dicts";
    paths = [
      hunspellDicts.pt_BR
      hunspellDicts.en_US
    ];
  };

  # The scripts run with `python3 -I`, which ignores PYTHONPATH: the modules
  # they import have to be inside the interpreter itself.
  python = python3.withPackages (ps: [
    ps.pygobject3
    ps.pillow
  ]);

  runtimeTools = [
    python
    glib # gdbus, gsettings
    wl-clipboard
    imagemagick
    poppler-utils
    ghostscript
    playerctl
    libnotify
    curl
    sqlite
    ocr
    quickshell # clippit-shell (the interface outside GNOME)
    wtype
    grim
    fd # file search where there is no tracker3 index
    pipewire # pw-record, for dictation
  ]
  ++ lib.optional withAI llama-cpp;

  scripts = [
    "clippit-correct"
    "clippit-busca"
    "clippit-historico"
    "clippit-doutor"
    "clippit-jogos-nvidia"
    "clippit-shell"
  ];

  extensionUuid = "clippit-corretor@clippit";
in
rustPlatform.buildRustPackage {
  pname = "clippit";
  inherit (cargoToml.workspace.package) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
      ../scripts
      ../assets
      ../gnome-extension
      ../quickshell
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = [
    "-p clippit-daemon"
    "-p clippit-popup"
    "-p clippit-dashboard"
    "-p clippit-wl"
  ];

  # The tests need a running session (clipboard, GTK).
  doCheck = false;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
    wrapGAppsHook4
    gobject-introspection
    rustPlatform.bindgenHook # tesseract/leptonica bindings
  ];

  buildInputs = [
    gtk4
    libadwaita
    glib
    ibus # IBus typelib, for the emoji names in clippit-busca
    ocr
    leptonica
    # tesseract's own links, which the final link needs too
    sqlite
    libarchive
    curl
    libx11
    libxcb
    libxkbcommon
    wayland
    hunspell
    cairo
    pango
  ];

  postInstall = ''
    for s in ${lib.concatStringsSep " " scripts}; do
      install -Dm755 scripts/$s $out/bin/$s
    done
    # patchShebangs leaves `env -S` lines alone.
    for s in ${lib.concatStringsSep " " scripts}; do
      sed -i '1s|^#!/usr/bin/env -S python3 -I$|#!${python.interpreter} -I|' $out/bin/$s
    done

    mkdir -p $out/share/gnome-shell/extensions
    cp -r gnome-extension/${extensionUuid} $out/share/gnome-shell/extensions/

    # The Quickshell interface finds the extension's icons at ../gnome-extension.
    mkdir -p $out/share/clippit
    cp -r quickshell $out/share/clippit/quickshell
    ln -s ../gnome-shell/extensions $out/share/clippit/gnome-extension

    install -Dm644 assets/logo_clippit.png $out/share/icons/hicolor/256x256/apps/clippit.png
    install -Dm644 assets/clippit.desktop $out/share/applications/clippit.desktop
    install -Dm644 assets/clippit-popup-autostart.desktop $out/share/clippit/clippit-popup-autostart.desktop
    substituteInPlace $out/share/clippit/clippit-popup-autostart.desktop \
      --replace-fail "Exec=clippit-popup" "Exec=$out/bin/clippit-popup"

    for unit in clippit.service clippit-doutor.service clippit-doutor.timer clippit-shell.service; do
      install -Dm644 assets/systemd/$unit $out/lib/systemd/user/$unit
    done
    substituteInPlace $out/lib/systemd/user/clippit.service $out/lib/systemd/user/clippit-doutor.service \
      $out/lib/systemd/user/clippit-shell.service \
      --replace-fail "%h/.local/bin" "$out/bin"
  '';

  dontWrapGApps = true;

  # Wrapped by hand so the scripts get the same environment as the binaries.
  # PATH is suffixed: whatever the user has installed still wins.
  postFixup = ''
    for f in $out/bin/*; do
      wrapProgram "$f" \
        "''${gappsWrapperArgs[@]}" \
        --suffix PATH : ${lib.makeBinPath runtimeTools}:$out/bin \
        --prefix LD_LIBRARY_PATH : ${
          lib.makeLibraryPath [
            hunspell
            libx11
            libxcb
            libxkbcommon
          ]
        } \
        --set-default CLIPPIT_HUNSPELL_DIR ${dictionaries}/share/hunspell \
        --set-default TESSDATA_PREFIX ${ocr}/share/tessdata \
        --set-default CLIPPIT_IBUS_DICTS ${ibus}/share/ibus/dicts \
        --set-default CLIPPIT_QUICKSHELL $out/share/clippit/quickshell
    done
  '';

  passthru = { inherit extensionUuid; };

  meta = {
    description = "Clipboard history, launcher, spell checker and local AI for GNOME on Wayland";
    homepage = "https://github.com/kaiolabs/Clippit";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "clippit-dashboard";
  };
}
